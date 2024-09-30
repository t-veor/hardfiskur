use std::time::Duration;

use hardfiskur_core::{
    board::{Board, Color, Move},
    move_gen::{MoveGenFlags, MoveVec},
};

use crate::{
    evaluation::evaluate,
    move_ordering::order_moves,
    score::Score,
    search_stats::SearchStats,
    transposition_table::{TranspositionEntry, TranspositionFlag, TranspositionTable},
};

pub struct SearchContext<'a> {
    pub board: &'a mut Board,
    pub allocated_time: Duration,
    pub stats: SearchStats,
    pub time_up: bool,

    pub tt: &'a mut TranspositionTable,
}

impl<'a> SearchContext<'a> {
    pub fn new(
        board: &'a mut Board,
        allocated_time: Duration,
        tt: &'a mut TranspositionTable,
    ) -> Self {
        Self {
            board,
            allocated_time,
            stats: SearchStats::new(),
            time_up: false,
            tt,
        }
    }

    pub fn is_time_up(&mut self) -> bool {
        if self.time_up {
            return true;
        }

        // Avoid syscalls a bit
        if self.stats.nodes_searched % 2048 != 0 {
            return false;
        }

        self.time_up = self.stats.search_started.elapsed() >= self.allocated_time;
        self.time_up
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

    if depth == 0 {
        return (quiescence_search(ctx, ply_from_root, alpha, beta), None);
    }

    let mut tt_move = None;
    match ctx.tt.get_entry(ctx.board.zobrist_hash()) {
        Some(entry) => {
            tt_move = entry.best_move;

            if entry.depth >= depth {
                ctx.stats.tt_hits += 1;

                let score = entry.get_score(ply_from_root);
                match entry.flag {
                    TranspositionFlag::Exact => {
                        return (score, entry.best_move);
                    }
                    TranspositionFlag::Lowerbound => beta = beta.min(score),
                    TranspositionFlag::Upperbound => alpha = alpha.max(score),
                }

                // Caused a cutoff? Return immediately
                if alpha >= beta {
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
            (-Score::mate_in_plies(ply_from_root), None)
        } else {
            (Score(0), None)
        };
    }

    // Handle repetitions & fifty-move rule
    if ctx
        .board
        .current_position_repeated_at_least(if ply_from_root > 2 { 1 } else { 2 })
        || ctx.board.halfmove_clock() >= 100
    {
        return (Score(0), None);
    }

    order_moves(ctx, tt_move, &mut legal_moves);

    let mut best_move = None;
    for m in legal_moves {
        ctx.board.push_move_unchecked(m);
        let eval = -simple_negamax_search(ctx, depth - 1, ply_from_root + 1, -beta, -alpha).0;
        ctx.board.pop_move();

        // Out of time, stop searching!
        if depth > 1 && ctx.is_time_up() {
            return (alpha, best_move);
        }

        if eval > alpha {
            alpha = eval;
            best_move = Some(m);
            tt_flag = TranspositionFlag::Exact;

            if alpha >= beta {
                tt_flag = TranspositionFlag::Lowerbound;
                ctx.stats.beta_cutoffs += 1;
                break;
            }
        }
    }

    ctx.tt.set(TranspositionEntry::new(
        ctx.board.zobrist_hash(),
        tt_flag,
        depth,
        alpha,
        best_move,
        ply_from_root,
    ));

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

pub fn iterative_deepening_search(
    board: &mut Board,
    allocated_time: Duration,
    transposition_table: &mut TranspositionTable,
) -> (Score, Option<Move>, SearchStats) {
    let mut ctx = SearchContext::new(board, allocated_time, transposition_table);

    let mut best_score = Score(0);
    let mut best_move = None;
    for depth in 1.. {
        let (score, m) = simple_negamax_search(&mut ctx, depth, 0, -Score::INF, Score::INF);

        if let Some(m) = m {
            best_score = score;
            best_move = Some(m);

            ctx.stats.depth = depth;
        }

        if ctx.is_time_up() {
            break;
        }
    }

    (best_score, best_move, ctx.stats)
}
