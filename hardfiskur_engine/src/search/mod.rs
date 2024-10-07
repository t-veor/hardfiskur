mod extensions;

use std::{
    sync::atomic::{AtomicBool, Ordering as AtomicOrdering},
    time::Instant,
};

use extensions::extensions;
use hardfiskur_core::{
    board::{Board, Color, Move, UCIMove},
    move_gen::{MoveGenFlags, MoveVec},
};

use crate::{
    diag,
    evaluation::evaluate,
    move_ordering::MoveOrderer,
    score::Score,
    search_limits::SearchLimits,
    search_result::SearchResult,
    search_stats::SearchStats,
    transposition_table::{TranspositionEntry, TranspositionFlag, TranspositionTable},
};

// In practice, we should never get to this search depth; however it avoids
// pathlogical behavior if the search function has a bug that immediately
// returns, for example.
const MAX_SEARCH_DEPTH: u32 = 256;

pub struct SearchContext<'a> {
    pub board: &'a mut Board,
    pub search_limits: SearchLimits,
    pub start_time: Instant,
    pub stats: SearchStats,
    pub time_up: bool,

    pub tt: &'a mut TranspositionTable,
    pub move_orderer: MoveOrderer,

    pub abort_flag: &'a AtomicBool,
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
}

pub fn simple_negamax_search(
    ctx: &mut SearchContext,
    depth: u32,
    ply_from_root: u32,
    mut alpha: Score,
    mut beta: Score,
    extension_count: u32,
) -> (Score, Option<Move>) {
    ctx.stats.nodes_searched += 1;
    ctx.stats.sel_depth = ctx.stats.sel_depth.max(ply_from_root);

    diag!(
        ctx.board,
        "Searching ply_from_root={ply_from_root} depth={depth} alpha={alpha} beta={beta}"
    );

    // Handle repetitions & fifty-move rule
    // This needs to go before the TT lookup, as otherwise entries in the table
    // may confuse it into thinking a repetition has a non-drawn score.
    if ctx
        .board
        .current_position_repeated_at_least(if ply_from_root >= 2 { 1 } else { 2 })
        || ctx.board.halfmove_clock() >= 100
    {
        diag!(ctx.board, "Found threefold rep or 50-move draw");
        return (Score(0), None);
    }

    let mut tt_move = None;
    match ctx.tt.get(ctx.board.zobrist_hash()) {
        Some(entry) => {
            tt_move = entry.best_move;

            diag!(ctx.board, "Got a TT hit for this position ({:?})", entry);

            if entry.depth >= depth {
                ctx.stats.tt_hits += 1;

                let score = entry.get_score(ply_from_root);
                match entry.flag {
                    TranspositionFlag::Exact => {
                        diag!(ctx.board, "Exact entry, returning stored score {score}");
                        return (score, entry.best_move);
                    }
                    TranspositionFlag::Upperbound => {
                        diag!(ctx.board, "Upperbound, lowering beta to {score}");
                        beta = beta.min(score)
                    }
                    TranspositionFlag::Lowerbound => {
                        diag!(ctx.board, "Lowerbound, raising alpha to {score}");
                        alpha = alpha.max(score);
                    }
                }

                // Caused a cutoff? Return immediately
                if alpha >= beta {
                    ctx.stats.beta_cutoffs += 1;
                    diag!(
                        ctx.board,
                        "TT entry caused a beta cutoff, returning {score} and best_move={}",
                        if let Some(m) = entry.best_move {
                            format!("{}", UCIMove::from(m))
                        } else {
                            "none".to_string()
                        }
                    );

                    return (score, entry.best_move);
                }
            }
        }
        None => (),
    }
    let mut tt_flag = TranspositionFlag::Upperbound;

    let (mut legal_moves, move_gen_result) = ctx.board.legal_moves_and_meta();

    // Handle checkmate/stalemate
    let in_check = move_gen_result.checker_count > 0;
    if legal_moves.is_empty() {
        return if move_gen_result.checker_count > 0 {
            diag!(ctx.board, "Found a checkmate");
            (-Score::mate_in_plies(ply_from_root), None)
        } else {
            diag!(ctx.board, "Found a stalemate");
            (Score(0), None)
        };
    }

    if depth == 0 {
        let score = quiescence_search(ctx, ply_from_root, alpha, beta);
        diag!(ctx.board, "Quiescence search returned {score}");
        return (score, None);
    }

    ctx.move_orderer
        .order_moves(&ctx.board, ply_from_root, tt_move, &mut legal_moves);

    let mut best_move_idx = None;
    let mut best_move = None;

    for (move_idx, m) in legal_moves.into_iter().enumerate() {
        ctx.board.push_move_unchecked(m);

        let extension = extensions(in_check, extension_count);
        let eval = -simple_negamax_search(
            ctx,
            depth - 1 + extension,
            ply_from_root + 1,
            -beta,
            -alpha,
            extension_count + extension,
        )
        .0;

        ctx.board.pop_move();

        diag!(
            ctx.board,
            "Move {}: {eval} (ply_from_root={ply_from_root})",
            UCIMove::from(m)
        );

        // Out of time, stop searching!
        if depth > 1 && ctx.should_exit_search() {
            diag!(ctx.board, "Out of time, returning {alpha}");

            return (alpha, best_move);
        }

        if eval > alpha {
            diag!(
                ctx.board,
                "Move {} raised alpha (prev_alpha={alpha}, score={eval}, beta={beta})",
                UCIMove::from(m)
            );

            alpha = eval;
            best_move = Some(m);
            best_move_idx = Some(move_idx);
            tt_flag = TranspositionFlag::Exact;

            if alpha >= beta {
                tt_flag = TranspositionFlag::Lowerbound;
                ctx.stats.beta_cutoffs += 1;
                ctx.stats.move_ordering.record_beta_cutoff(move_idx);

                diag!(
                    ctx.board,
                    "Fail-high: move {} caused a beta cutoff! Returning {beta} (depth={depth})",
                    UCIMove::from(m)
                );

                // Update killer moves
                ctx.move_orderer.store_killer(ply_from_root, m);

                ctx.tt.set(
                    ctx.board.zobrist_hash(),
                    TranspositionEntry::new(tt_flag, depth, beta, best_move, ply_from_root),
                );
                return (beta, best_move);
            }
        }
    }

    if tt_flag == TranspositionFlag::Lowerbound {
        diag!(
            ctx.board,
            "Fail-low: None of the moves were better than alpha={alpha}, returning alpha (depth={depth}, best_move={})",
            if let Some(m) = best_move {
                format!("{}", UCIMove::from(m))
            } else {
                "none".to_string()
            }
        );
    } else {
        diag!(
            ctx.board,
            "Exact: Returning score={alpha} (depth={depth}, best_move={})",
            if let Some(m) = best_move {
                format!("{}", UCIMove::from(m))
            } else {
                "none".to_string()
            }
        )
    }

    ctx.tt.set(
        ctx.board.zobrist_hash(),
        TranspositionEntry::new(tt_flag, depth, alpha, best_move, ply_from_root),
    );

    if let Some(i) = best_move_idx {
        ctx.stats.move_ordering.record_best_move(i);
    }

    (alpha, best_move)
}

pub fn quiescence_search(
    ctx: &mut SearchContext,
    ply_from_root: u32,
    mut alpha: Score,
    beta: Score,
) -> Score {
    ctx.stats.nodes_searched += 1;
    ctx.stats.quiescence_nodes += 1;

    let mut capturing_moves = MoveVec::new();
    ctx.board
        .legal_moves_ex(MoveGenFlags::GEN_CAPTURES, &mut capturing_moves);

    let stand_pat_score = {
        let score = evaluate(ctx.board);

        match ctx.board.to_move() {
            Color::White => score,
            Color::Black => -score,
        }
    };

    if stand_pat_score >= beta {
        // Return beta -- opponent would not have played the previous move to
        // get here
        ctx.stats.beta_cutoffs += 1;
        return beta;
    }

    alpha = alpha.max(stand_pat_score);

    ctx.move_orderer
        .order_moves(&ctx.board, ply_from_root, None, &mut capturing_moves);

    let mut best_move_idx = None;

    for (move_idx, m) in capturing_moves.into_iter().enumerate() {
        ctx.board.push_move_unchecked(m);
        let score = -quiescence_search(ctx, ply_from_root + 1, -beta, -alpha);
        ctx.board.pop_move();

        if score >= beta {
            ctx.stats.beta_cutoffs += 1;
            ctx.stats.move_ordering.record_beta_cutoff(move_idx);

            return beta;
        }

        if score > alpha {
            alpha = score;
            best_move_idx = Some(move_idx);
        }
    }

    if let Some(i) = best_move_idx {
        ctx.stats.move_ordering.record_best_move(i);
    }

    return alpha;
}

pub fn iterative_deepening_search(mut ctx: SearchContext) -> SearchResult {
    let mut best_score = Score(0);
    let mut best_move = None;

    for depth in 1..=(ctx.search_limits.depth.min(MAX_SEARCH_DEPTH)) {
        let (score, m) = simple_negamax_search(&mut ctx, depth, 0, -Score::INF, Score::INF, 0);

        if let Some(m) = m {
            best_score = score;
            best_move = Some(m);

            ctx.stats.depth = depth;

            // Already found a mate, don't need to look any further -- although,
            // don't trust mate scores that are greater than the current depth,
            // as they may be from the TT or extensions
            if let Some(signed_plies) = best_score.as_mate_in_plies() {
                if signed_plies.abs() as u32 <= depth {
                    break;
                }
            }
        }

        // TODO: Do this properly, e.g. by providing a listener to feed this
        // information to
        if depth > 1 && ctx.stats.nodes_searched > 4096 {
            let pv = ctx.tt.extract_pv(ctx.board);
            print!("info depth {depth} nodes {} ", ctx.stats.nodes_searched);
            if let Some(mate) = score.as_mate_in() {
                print!("score mate {mate} ");
            } else if let Some(cp) = score.as_centipawns() {
                print!("score cp {cp} ")
            }
            print!("hashfull {} ", ctx.tt.occupancy());
            print!("pv ",);
            for m in pv {
                print!("{} ", UCIMove::from(m));
            }
            println!();

            let pv_nodes = ctx
                .stats
                .move_ordering
                .pv_node_best_move_idxs
                .iter()
                .sum::<u32>();
            let cut_nodes = ctx
                .stats
                .move_ordering
                .beta_cutoff_move_idxs
                .iter()
                .sum::<u32>();

            println!(
                "info string pv_node_proportions {:?}",
                ctx.stats
                    .move_ordering
                    .pv_node_best_move_idxs
                    .iter()
                    .map(|&x| x as u64 * 1000 / pv_nodes.max(1) as u64)
                    .collect::<Vec<_>>()
            );
            println!(
                "info string beta_node_proportions {:?}",
                ctx.stats
                    .move_ordering
                    .beta_cutoff_move_idxs
                    .iter()
                    .map(|&x| x as u64 * 1000 / cut_nodes.max(1) as u64)
                    .collect::<Vec<_>>()
            );
        }

        if ctx.should_exit_search() {
            break;
        }
    }

    SearchResult {
        score: best_score,
        best_move,
        stats: ctx.stats,
        elapsed: ctx.start_time.elapsed(),
        aborted: ctx.abort_flag.load(AtomicOrdering::Relaxed),
        hash_full: ctx.tt.occupancy(),
    }
}
