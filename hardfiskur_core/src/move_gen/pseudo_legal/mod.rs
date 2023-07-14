use super::{MoveGenMasks, MoveGenerator};
use crate::board::{Bitboard, Move, PieceType, Square};
pub use pawn_moves::{black_pawn_attacks, white_pawn_attacks};

mod pawn_moves;

impl<'board, 'moves> MoveGenerator<'board, 'moves> {
    pub(super) fn pseudo_legal_moves(&mut self, masks: &MoveGenMasks) {
        // ignore king moves and en passant, those always have to be treated
        // differently by legal move generation
        self.pseudo_legal_pawn_moves(masks);
        self.pseudo_legal_knight_moves(masks);
        self.pseudo_legal_bishop_moves(masks);
        self.pseudo_legal_rook_moves(masks);
        self.pseudo_legal_queen_moves(masks);
    }

    // Works for all pieces but pawns
    // get_attack_pattern should take the currently occupied squares and the
    // source square and return all possible moves from there
    fn generic_pseudo_legal_moves<F>(
        &mut self,
        masks: &MoveGenMasks,
        piece_type: PieceType,
        get_attack_pattern: F,
    ) where
        F: Fn(Bitboard, Square) -> Bitboard,
    {
        let piece = piece_type.with_color(self.to_move);
        let movable_pieces = self.board[piece] & masks.movable;
        let capturable_pieces = self.board[self.to_move.flip()] & masks.capture;
        let pushable_squares = self.empty & masks.push;

        for from in movable_pieces.squares() {
            let attack_pattern = get_attack_pattern(self.occupied, from);

            let pushes = attack_pattern & pushable_squares;
            let captures = attack_pattern & capturable_pieces;

            for to in pushes.squares() {
                self.out_moves.push(Move::builder(from, to, piece).build());
            }

            for to in captures.squares() {
                self.out_moves.push(
                    Move::builder(from, to, piece)
                        .captures(
                            self.board
                                .piece_with_color_at(self.to_move.flip(), to)
                                .unwrap(),
                        )
                        .build(),
                );
            }
        }
    }

    fn pseudo_legal_knight_moves(&mut self, masks: &MoveGenMasks) {
        self.generic_pseudo_legal_moves(masks, PieceType::Knight, |_, from| {
            self.lookups.get_knight_moves(from)
        });
    }

    fn pseudo_legal_bishop_moves(&mut self, masks: &MoveGenMasks) {
        self.generic_pseudo_legal_moves(masks, PieceType::Bishop, |occupied, from| {
            self.lookups.get_bishop_attacks(occupied, from)
        });
    }

    fn pseudo_legal_rook_moves(&mut self, masks: &MoveGenMasks) {
        self.generic_pseudo_legal_moves(masks, PieceType::Rook, |occupied, from| {
            self.lookups.get_rook_attacks(occupied, from)
        });
    }

    fn pseudo_legal_queen_moves(&mut self, masks: &MoveGenMasks) {
        self.generic_pseudo_legal_moves(masks, PieceType::Queen, |occupied, from| {
            self.lookups.get_queen_attacks(occupied, from)
        });
    }
}
