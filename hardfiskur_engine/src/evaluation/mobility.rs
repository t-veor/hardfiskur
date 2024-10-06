use hardfiskur_core::board::{Color, Piece, PieceType, Square};

use super::EvalContext;

impl<'a> EvalContext<'a> {
    pub fn mobility_score(&self, piece: Piece, square: Square) -> i32 {
        let opponent_pawn_attacks = match piece.color() {
            Color::White => self.black_pawn_attacks,
            Color::Black => self.white_pawn_attacks,
        };

        let mobility_bitboard = match piece.piece_type() {
            // Don't worry about pawn mobility and king mobility here
            PieceType::Pawn | PieceType::King => return 0,

            PieceType::Knight => self.lookups.get_knight_moves(square),
            PieceType::Bishop => self.lookups.get_bishop_attacks(self.occupied, square),
            PieceType::Rook => self.lookups.get_rook_attacks(self.occupied, square),
            PieceType::Queen => self.lookups.get_queen_attacks(self.occupied, square),
        }
        .without(self.board.get_bitboard_for_color(piece.color()))
        .without(opponent_pawn_attacks);

        mobility_bitboard.pop_count() as i32 * self.mobility_weight(piece.piece_type())
    }

    fn mobility_weight(&self, piece_type: PieceType) -> i32 {
        match piece_type {
            PieceType::Pawn | PieceType::King => 0,

            PieceType::Knight => 25,
            PieceType::Bishop => 20,
            PieceType::Rook => 10,
            PieceType::Queen => 5,
        }
    }
}
