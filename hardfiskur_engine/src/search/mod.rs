mod extensions;

use std::{
    sync::atomic::{AtomicBool, Ordering as AtomicOrdering},
    time::Instant,
};

use extensions::extensions;
use hardfiskur_core::{
    board::{Board, Move, UCIMove},
    move_gen::{MoveGenFlags, MoveVec},
};

use crate::{
    evaluation::evaluate,
    move_ordering::MoveOrderer,
    parameters::MAX_DEPTH,
    score::Score,
    search_limits::SearchLimits,
    search_result::SearchResult,
    search_stats::SearchStats,
    transposition_table::{TranspositionEntry, TranspositionFlag, TranspositionTable},
};

pub struct SearchContext<'a> {
    pub board: &'a mut Board,
    pub search_limits: SearchLimits,
    pub start_time: Instant,
    pub stats: SearchStats,
    pub time_up: bool,

    pub tt: &'a mut TranspositionTable,
    pub move_orderer: MoveOrderer,

    pub abort_flag: &'a AtomicBool,

    pub best_root_move: Option<Move>,
}

impl<'a> SearchContext<'a> {
    pub fn new(
        board: &'a mut Board,
        search_limits: SearchLimits,
        tt: &'a mut TranspositionTable,
        abort_flag: &'a AtomicBool,
    ) -> Self {
        Self {
            board,
            search_limits,
            start_time: Instant::now(),
            stats: SearchStats::default(),
            time_up: false,
            tt,
            move_orderer: MoveOrderer::new(),
            abort_flag,
            best_root_move: None,
        }
    }

    pub fn should_exit_search(&mut self) -> bool {
        self.is_time_up() || self.over_node_budget()
    }

    pub fn is_time_up(&mut self) -> bool {
        if self.time_up {
            return true;
        }

        // Avoid syscalls a bit
        if self.stats.nodes_searched % 2048 != 0 {
            return false;
        }

        self.time_up = self.start_time.elapsed() >= self.search_limits.allocated_time
            || self.abort_flag.load(AtomicOrdering::Relaxed);

        self.time_up
    }

    pub fn over_node_budget(&self) -> bool {
        self.stats.nodes_searched >= self.search_limits.node_budget
    }

    pub fn negamax<const ROOT: bool>(
        &mut self,
        depth: i16,
        ply_from_root: u16,
        mut alpha: Score,
        mut beta: Score,
        extension_count: i16,
    ) -> Score {
        self.stats.nodes_searched += 1;
        self.stats.sel_depth = self.stats.sel_depth.max(ply_from_root);

        // handle repetitions & fifty-move rule
        // this needs to go before the tt lookup, as otherwise entries in the table
        // may confuse it into thinking a repetition has a non-drawn score.
        if self
            .board
            .current_position_repeated_at_least(if ply_from_root >= 2 { 1 } else { 2 })
            || self.board.halfmove_clock() >= 100
        {
            return Score(0);
        }

        let mut tt_move = None;
        match self.tt.get(self.board.zobrist_hash()) {
            Some(entry) => {
                tt_move = entry.best_move;

                if entry.depth >= depth {
                    self.stats.tt_hits += 1;

                    let score = entry.get_score(ply_from_root);
                    match entry.flag {
                        TranspositionFlag::Exact => {
                            if ROOT {
                                self.best_root_move = entry.best_move;
                            }
                            return score;
                        }
                        TranspositionFlag::Upperbound => beta = beta.min(score),
                        TranspositionFlag::Lowerbound => alpha = alpha.max(score),
                    }

                    // Caused a cutoff? Return immediately
                    if alpha >= beta {
                        self.stats.beta_cutoffs += 1;

                        if ROOT {
                            self.best_root_move = entry.best_move;
                        }

                        return score;
                    }
                }
            }
            None => (),
        }
        let mut tt_flag = TranspositionFlag::Upperbound;

        let (mut legal_moves, move_gen_result) = self.board.legal_moves_and_meta();

        // Handle checkmate/stalemate
        let in_check = move_gen_result.checker_count > 0;
        if legal_moves.is_empty() {
            return if move_gen_result.checker_count > 0 {
                // Checkmate
                -Score::mate_in_plies(ply_from_root)
            } else {
                // Stalemate
                Score(0)
            };
        }

        // TODO: Try not transitioning into the quiescence search if in check
        if depth <= 0 {
            return self.quiescence_search(ply_from_root, alpha, beta);
        }

        self.move_orderer
            .order_moves(self.board, ply_from_root, tt_move, &mut legal_moves);

        let mut best_score = -Score::INF;
        let mut best_move = None;
        let mut best_move_idx = None;

        for (move_idx, m) in legal_moves.into_iter().enumerate() {
            self.board.push_move_unchecked(m);

            let extension = extensions(in_check, extension_count);
            let eval = -self.negamax::<false>(
                depth - 1 + extension,
                ply_from_root + 1,
                -beta,
                -alpha,
                extension_count + extension,
            );

            self.board.pop_move();

            // Out of time, stop searching!
            if depth > 1 && self.should_exit_search() {
                return best_score;
            }

            if eval > best_score {
                best_score = eval;
                best_move = Some(m);
                best_move_idx = Some(move_idx);

                if ROOT {
                    self.best_root_move = Some(m);
                }

                if eval >= beta {
                    tt_flag = TranspositionFlag::Lowerbound;
                    self.stats.beta_cutoffs += 1;
                    self.stats.move_ordering.record_beta_cutoff(move_idx);

                    // Update killer moves
                    self.move_orderer.store_killer(ply_from_root, m);
                    break;
                }

                if eval > alpha {
                    tt_flag = TranspositionFlag::Exact;
                    alpha = eval;
                }
            }
        }

        self.tt.set(
            self.board.zobrist_hash(),
            TranspositionEntry::new(tt_flag, depth, best_score, best_move, ply_from_root),
        );

        if let Some(i) = best_move_idx {
            self.stats.move_ordering.record_best_move(i);
        }

        best_score
    }

    pub fn quiescence_search(
        &mut self,
        ply_from_root: u16,
        mut alpha: Score,
        beta: Score,
    ) -> Score {
        self.stats.nodes_searched += 1;
        self.stats.quiescence_nodes += 1;

        let stand_pat_score = evaluate(self.board);

        if stand_pat_score >= beta {
            self.stats.beta_cutoffs += 1;
            return stand_pat_score;
        }

        let mut best_score = stand_pat_score;
        alpha = alpha.max(stand_pat_score);

        let mut capturing_moves = MoveVec::new();
        self.board
            .legal_moves_ex(MoveGenFlags::GEN_CAPTURES, &mut capturing_moves);

        self.move_orderer
            .order_moves(self.board, ply_from_root, None, &mut capturing_moves);

        let mut best_move_idx = None;

        for (move_idx, m) in capturing_moves.into_iter().enumerate() {
            self.board.push_move_unchecked(m);
            let eval = -self.quiescence_search(ply_from_root + 1, -beta, -alpha);
            self.board.pop_move();

            if eval > best_score {
                best_score = eval;
                best_move_idx = Some(move_idx);

                if eval >= beta {
                    self.stats.beta_cutoffs += 1;
                    self.stats.move_ordering.record_beta_cutoff(move_idx);

                    // Update killer moves
                    self.move_orderer.store_killer(ply_from_root, m);
                    break;
                }

                if eval > alpha {
                    alpha = eval;
                }
            }
        }

        if let Some(i) = best_move_idx {
            self.stats.move_ordering.record_best_move(i);
        }

        return best_score;
    }

    pub fn iterative_deepening_search(mut self) -> SearchResult {
        let mut best_score = Score(0);
        let mut best_move = None;

        for depth in 1..=(self.search_limits.depth.min(MAX_DEPTH)) {
            let score = self.negamax::<true>(depth, 0, -Score::INF, Score::INF, 0);

            if let Some(m) = self.best_root_move.take() {
                // If we did not even get a root move from a partial search then
                // we can't accept its results.
                best_score = score;
                best_move = Some(m);

                // Already found a mate, don't need to look any further -- although,
                // don't trust mate scores that are greater than the current depth,
                // as they may be from the TT or extensions
                if let Some(signed_plies) = best_score.as_mate_in_plies() {
                    if signed_plies.abs() <= depth as i32 {
                        break;
                    }
                }
            }

            self.stats.depth = depth as _;

            // TODO: Do this properly, e.g. by providing a listener to feed this
            // information to
            if depth > 1 && self.stats.nodes_searched > 4096 {
                let pv = self.tt.extract_pv(self.board);
                print!("info depth {depth} nodes {} ", self.stats.nodes_searched);
                if let Some(mate) = score.as_mate_in() {
                    print!("score mate {mate} ");
                } else if let Some(cp) = score.as_centipawns() {
                    print!("score cp {cp} ")
                }
                print!("hashfull {} ", self.tt.occupancy());
                print!("pv ",);
                for m in pv {
                    print!("{} ", UCIMove::from(m));
                }
                println!();

                let pv_nodes = self
                    .stats
                    .move_ordering
                    .pv_node_best_move_idxs
                    .iter()
                    .sum::<u64>();
                let cut_nodes = self
                    .stats
                    .move_ordering
                    .beta_cutoff_move_idxs
                    .iter()
                    .sum::<u64>();

                println!(
                    "info string pv_node_proportions {:?}",
                    self.stats
                        .move_ordering
                        .pv_node_best_move_idxs
                        .iter()
                        .map(|&x| x * 1000 / pv_nodes.max(1))
                        .collect::<Vec<_>>()
                );
                println!(
                    "info string beta_node_proportions {:?}",
                    self.stats
                        .move_ordering
                        .beta_cutoff_move_idxs
                        .iter()
                        .map(|&x| x * 1000 / cut_nodes.max(1))
                        .collect::<Vec<_>>()
                );
            }

            if self.should_exit_search() {
                break;
            }
        }

        SearchResult {
            score: best_score,
            best_move,
            stats: self.stats,
            elapsed: self.start_time.elapsed(),
            aborted: self.abort_flag.load(AtomicOrdering::Relaxed),
            hash_full: self.tt.occupancy(),
        }
    }
}
