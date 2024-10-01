use std::{
    sync::atomic::{AtomicBool, Ordering as AtomicOrdering},
    time::Instant,
};

use hardfiskur_core::{
    board::{Board, Color, Move},
    move_gen::{MoveGenFlags, MoveVec},
};

use crate::{
    diag,
    evaluation::evaluate,
    move_ordering::order_moves,
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
) -> (Score, Option<Move>) {
    ctx.stats.nodes_searched += 1;

    diag!("info string Searching ply_from_root={ply_from_root} depth={depth} alpha={alpha} beta={beta}");

    if depth == 0 {
        let score = quiescence_search(ctx, ply_from_root, alpha, beta);
        diag!("info string Quiescence search returned {score}");
        return (score, None);
    }

    let mut tt_move = None;
    match ctx.tt.get_entry(ctx.board.zobrist_hash()) {
        Some(entry) => {
            diag!(
                "info string Got a TT hit for this position (prev_depth={}, best_move={})",
                entry.depth,
                if let Some(m) = entry.best_move {
                    format!("{m}")
                } else {
                    "none".to_string()
                }
            );

            tt_move = entry.best_move;

            if entry.depth >= depth {
                ctx.stats.tt_hits += 1;

                let score = entry.get_score(ply_from_root);
                match entry.flag {
                    TranspositionFlag::Exact => {
                        diag!("info string Exact entry, returning stored score {score}");
                        return (score, entry.best_move);
                    }
                    TranspositionFlag::Upperbound => {
                        diag!("info string Upperbound, lowering beta to {score}");
                        beta = beta.min(score)
                    }
                    TranspositionFlag::Lowerbound => {
                        diag!("info string Lowerbound, raising alpha to {score}");
                        alpha = alpha.max(score)
                    }
                }

                // Caused a cutoff? Return immediately
                if alpha >= beta {
                    ctx.stats.beta_cutoffs += 1;
                    diag!(
                        "info TT entry caused a beta cutoff, returning {score} and best_move={}",
                        if let Some(m) = entry.best_move {
                            format!("{m}")
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
    if legal_moves.is_empty() {
        return if move_gen_result.checker_count > 0 {
            diag!("info string Found a checkmate");
            (-Score::mate_in_plies(ply_from_root), None)
        } else {
            diag!("info string Found a stalemate");
            (Score(0), None)
        };
    }

    // Handle repetitions & fifty-move rule
    if ctx
        .board
        .current_position_repeated_at_least(if ply_from_root > 2 { 1 } else { 2 })
        || ctx.board.halfmove_clock() >= 100
    {
        diag!("info string Found a threefold repetition or 50-move draw");
        return (Score(0), None);
    }

    order_moves(ctx, tt_move, &mut legal_moves);

    let mut best_move = None;
    for m in legal_moves {
        diag!("info string Considering move {m}...");

        ctx.board.push_move_unchecked(m);
        let eval = -simple_negamax_search(ctx, depth - 1, ply_from_root + 1, -beta, -alpha).0;
        ctx.board.pop_move();

        // Out of time, stop searching!
        if depth > 1 && ctx.should_exit_search() {
            diag!("info string Out of time, returning {alpha}");
            return (alpha, best_move);
        }

        diag!("info string Received {eval} (ply_from_root={ply_from_root}");

        if eval > alpha {
            diag!(
                "info string Move {m} raised alpha (prev_alpha={alpha}, score={eval} beta={beta})"
            );

            alpha = eval;
            best_move = Some(m);
            tt_flag = TranspositionFlag::Exact;

            if alpha >= beta {
                tt_flag = TranspositionFlag::Lowerbound;
                ctx.stats.beta_cutoffs += 1;

                diag!("info string Fail-high: move {m} caused a beta cutoff! Returning {beta}");

                ctx.tt.set(
                    ctx.board.zobrist_hash(),
                    TranspositionEntry::new(tt_flag, depth, beta, best_move, ply_from_root),
                );
                return (beta, best_move);
            }
        }
    }

    #[cfg(feature = "hardfiskur_emit_diagnostics")]
    if tt_flag == TranspositionFlag::Lowerbound {
        diag!(
            "info string Fail-low: None of the moves were better than alpha={alpha}, returning alpha (best_move={})",
            if let Some(m) = best_move {
                format!("{m}")
            } else {
                "none".to_string()
            }
        );
    } else {
        diag!(
            "info string Exact: Returning score={alpha} (best_move={})",
            if let Some(m) = best_move {
                format!("{m}")
            } else {
                "none".to_string()
            }
        )
    }

    ctx.tt.set(
        ctx.board.zobrist_hash(),
        TranspositionEntry::new(tt_flag, depth, alpha, best_move, ply_from_root),
    );

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

    order_moves(ctx, None, &mut capturing_moves);

    for m in capturing_moves {
        ctx.board.push_move_unchecked(m);
        let score = -quiescence_search(ctx, ply_from_root + 1, -beta, -alpha);
        ctx.board.pop_move();

        if score >= beta {
            ctx.stats.beta_cutoffs += 1;
            return beta;
        }
        alpha = alpha.max(score);
    }

    return alpha;
}

pub fn iterative_deepening_search(mut ctx: SearchContext) -> SearchResult {
    let mut best_score = Score(0);
    let mut best_move = None;

    for depth in 1.. {
        diag!("info string ================================================================================");
        diag!("info string Beginning new search iteration with depth {depth}");
        diag!("info string ================================================================================");

        let (score, m) = simple_negamax_search(&mut ctx, depth, 0, -Score::INF, Score::INF);

        diag!(
            "info string Received score={score}, move={} from depth={depth} search",
            if let Some(m) = m {
                format!("{m}")
            } else {
                "none".to_string()
            }
        );

        if let Some(m) = m {
            best_score = score;
            best_move = Some(m);

            ctx.stats.depth = depth;

            // Already found a mate, don't need to look any further
            if best_score.is_mate() {
                break;
            }
        }

        if ctx.should_exit_search() || depth >= ctx.search_limits.depth {
            break;
        }
    }

    SearchResult {
        score: best_score,
        best_move,
        stats: ctx.stats,
        elapsed: ctx.start_time.elapsed(),
        aborted: ctx.abort_flag.load(AtomicOrdering::Relaxed),
    }
}
