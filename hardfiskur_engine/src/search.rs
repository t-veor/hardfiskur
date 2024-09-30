use std::sync::atomic::AtomicBool;

use hardfiskur_core::{
    board::{Board, Color, Move},
    move_gen::{MoveGenFlags, MoveVec},
};

use crate::{
    evaluation::evaluate, move_ordering::order_moves, score::Score, search_stats::SearchStats,
};

pub struct SearchContext<'a> {
    pub board: &'a mut Board,
    pub stats: SearchStats,
    pub stop_flag: AtomicBool,
}

impl<'a> SearchContext<'a> {
    pub fn new(board: &'a mut Board, stop_flag: AtomicBool) -> Self {
        Self {
            board,
            stats: SearchStats::new(),
            stop_flag,
        }
    }

    pub fn search_cancelled(&self) -> bool {
        self.stop_flag.load(std::sync::atomic::Ordering::Relaxed)
    }
}

pub fn simple_negamax_search(
    ctx: &mut SearchContext,
    depth: u32,
    ply_from_root: u32,
    mut alpha: Score,
    beta: Score,
) -> (Score, Option<Move>) {
    ctx.stats.nodes_searched += 1;

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

    if depth == 0 {
        return (quiescence_search(ctx, ply_from_root, alpha, beta), None);
    }

    order_moves(ctx.board, &mut legal_moves);

    let mut best_move = None;
    for m in legal_moves {
        ctx.board.push_move_unchecked(m);
        let eval = -simple_negamax_search(ctx, depth - 1, ply_from_root + 1, -beta, -alpha).0;
        ctx.board.pop_move();

        if eval > alpha {
            alpha = eval;
            best_move = Some(m);

            if alpha >= beta {
                ctx.stats.beta_cutoffs += 1;
                break;
            }
        }
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

    order_moves(ctx.board, &mut capturing_moves);

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

pub fn simple_search(board: &mut Board) -> (Score, Option<Move>, SearchStats) {
    let mut ctx = SearchContext::new(board, AtomicBool::new(false));
    ctx.stats.depth = 4;

    let (score, best_move) = simple_negamax_search(&mut ctx, 4, 0, -Score::INF, Score::INF);

    (score, best_move, ctx.stats)
}
