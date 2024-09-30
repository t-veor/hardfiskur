use std::time::Instant;

use hardfiskur_core::{
    board::{Board, Color, Move},
    move_gen::{MoveGenFlags, MoveVec},
};

use crate::{evaluation::evaluate, move_ordering::order_moves, score::Score};

#[derive(Debug)]
pub struct SearchStats {
    pub depth: u32,
    pub search_started: Instant,
    pub nodes_searched: u64,
    pub quiescence_nodes: u64,
    pub beta_cutoffs: u32,
}

impl SearchStats {
    pub fn new() -> Self {
        Self {
            depth: 0,
            search_started: Instant::now(),
            nodes_searched: 0,
            quiescence_nodes: 0,
            beta_cutoffs: 0,
        }
    }
}

pub fn simple_negamax_search(
    board: &mut Board,
    depth: u32,
    ply_from_root: u32,
    mut alpha: Score,
    beta: Score,
    stats: &mut SearchStats,
) -> (Score, Option<Move>) {
    stats.nodes_searched += 1;

    let (mut legal_moves, move_gen_result) = board.legal_moves_and_meta();

    // Handle checkmate/stalemate
    if legal_moves.is_empty() {
        return if move_gen_result.checker_count > 0 {
            (-Score::mate_in_plies(ply_from_root), None)
        } else {
            (Score(0), None)
        };
    }

    // Handle repetitions & fifty-move rule
    if board.current_position_repeated_at_least(if ply_from_root > 2 { 1 } else { 2 })
        || board.halfmove_clock() >= 100
    {
        return (Score(0), None);
    }

    if depth == 0 {
        return (
            quiescence_search(board, ply_from_root, alpha, beta, stats),
            None,
        );
    }

    order_moves(board, &mut legal_moves);

    let mut best_move = None;
    for m in legal_moves {
        board.push_move_unchecked(m);
        let eval =
            -simple_negamax_search(board, depth - 1, ply_from_root + 1, -beta, -alpha, stats).0;
        board.pop_move();

        if eval > alpha {
            alpha = eval;
            best_move = Some(m);

            if alpha >= beta {
                stats.beta_cutoffs += 1;
                break;
            }
        }
    }

    (alpha, best_move)
}

pub fn quiescence_search(
    board: &mut Board,
    ply_from_root: u32,
    mut alpha: Score,
    beta: Score,
    stats: &mut SearchStats,
) -> Score {
    stats.nodes_searched += 1;
    stats.quiescence_nodes += 1;

    let mut capturing_moves = MoveVec::new();
    board.legal_moves_ex(MoveGenFlags::GEN_CAPTURES, &mut capturing_moves);

    let stand_pat_score = {
        let score = evaluate(board);

        match board.to_move() {
            Color::White => score,
            Color::Black => -score,
        }
    };

    if stand_pat_score >= beta {
        // Return beta -- opponent would not have played the previous move to
        // get here
        stats.beta_cutoffs += 1;
        return beta;
    }

    alpha = alpha.max(stand_pat_score);

    order_moves(board, &mut capturing_moves);

    for m in capturing_moves {
        board.push_move_unchecked(m);
        let score = -quiescence_search(board, ply_from_root + 1, -beta, -alpha, stats);
        board.pop_move();

        if score >= beta {
            stats.beta_cutoffs += 1;
            return beta;
        }
        alpha = alpha.max(score);
    }

    return alpha;
}

pub fn simple_search(board: &mut Board) -> (Score, Option<Move>, SearchStats) {
    let mut search_stats = SearchStats::new();
    search_stats.depth = 4;

    let (score, best_move) =
        simple_negamax_search(board, 4, 0, -Score::INF, Score::INF, &mut search_stats);

    (score, best_move, search_stats)
}
