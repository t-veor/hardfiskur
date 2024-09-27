use hardfiskur_core::board::{Board, Move};

use crate::evaluation::piece_tables::material_score;

pub fn order_moves(board: &Board, moves: &mut [Move]) {
    let last_moved_to = board.last_move().map(|m| m.to_square());

    moves.sort_by_cached_key(|m| {
        let mut score = 0;
        let move_piece = m.piece();
        let captured_piece = m.captured_piece();

        if let Some(captured_piece) = captured_piece {
            // Reward moves that recapture the piece that just moved highly
            let weight = if Some(m.to_square()) == last_moved_to {
                100
            } else {
                10
            };

            score += weight * material_score(captured_piece.piece_type())
                - material_score(move_piece.piece_type());
        }

        if let Some(promotion) = m.promotion() {
            score += material_score(promotion.piece_type());
        }

        if board.attacked_by_pawn(board.to_move(), m.to_square()) {
            score -= material_score(move_piece.piece_type());
        }

        -score
    });
}
