use hardfiskur_core::board::{Board, Color, Move};

use crate::{evaluation::evaluate, move_ordering::order_moves, score::Score};

#[derive(Debug, Default)]
pub struct SearchStats {
    pub nodes_searched: u64,
    pub beta_cutoffs: u64,
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

    let to_move = board.to_move();

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
        let score = evaluate(board);

        let eval = match to_move {
            Color::White => score,
            Color::Black => -score,
        };
        return (eval, None);
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

pub fn simple_search(board: &mut Board) -> Option<Move> {
    let mut search_stats = SearchStats::default();

    let (score, best_move) =
        simple_negamax_search(board, 4, 0, -Score::INF, Score::INF, &mut search_stats);
    // match board.to_move() {
    //     Color::White => println!("{}", score),
    //     Color::Black => println!("{}", -score),
    // }
    // println!("{search_stats:?}");
    best_move
}
