use super::{MoveGenMasks, MoveGenerator};
use crate::board::{Bitboard, Move, PieceType, Square};
pub use pawn_moves::{black_pawn_attacks, white_pawn_attacks};

mod pawn_moves;

impl<'board, 'moves> MoveGenerator<'board, 'moves> {
    pub(super) fn pseudo_legal_moves(&mut self, masks: &MoveGenMasks) {
        // ignore king moves and castling moves, those always have to be treated
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

#[cfg(test)]
mod test {
    use crate::{
        board::{BoardRepr, Color, Piece},
        move_gen::MoveVec,
        test_utils::assert_in_any_order,
    };

    use super::*;

    fn test_position() -> BoardRepr {
        "
            ...qk...
            ....p...
            b.n..r..
            PqN.n..R
            .P....Q.
            ...Nb.p.
            .B.P...P
            ....KB..
        "
        .parse()
        .unwrap()
    }

    fn white_pawn_moves() -> Vec<Move> {
        vec![
            Move::builder(Square::H2, Square::H3, Piece::WHITE_PAWN).build(),
            Move::builder(Square::H2, Square::H4, Piece::WHITE_PAWN)
                .is_double_pawn_push()
                .build(),
            //
            Move::builder(Square::D2, Square::E3, Piece::WHITE_PAWN)
                .captures(Piece::BLACK_BISHOP)
                .build(),
            Move::builder(Square::H2, Square::G3, Piece::WHITE_PAWN)
                .captures(Piece::BLACK_PAWN)
                .build(),
        ]
    }

    fn black_pawn_moves() -> Vec<Move> {
        vec![
            Move::builder(Square::E7, Square::E6, Piece::BLACK_PAWN).build(),
            Move::builder(Square::G3, Square::G2, Piece::BLACK_PAWN).build(),
            //
            Move::builder(Square::G3, Square::H2, Piece::BLACK_PAWN)
                .captures(Piece::WHITE_PAWN)
                .build(),
        ]
    }

    fn white_knight_moves() -> Vec<Move> {
        vec![
            Move::builder(Square::C5, Square::B7, Piece::WHITE_KNIGHT).build(),
            Move::builder(Square::C5, Square::D7, Piece::WHITE_KNIGHT).build(),
            Move::builder(Square::C5, Square::E6, Piece::WHITE_KNIGHT).build(),
            Move::builder(Square::C5, Square::E4, Piece::WHITE_KNIGHT).build(),
            Move::builder(Square::C5, Square::B3, Piece::WHITE_KNIGHT).build(),
            Move::builder(Square::C5, Square::A4, Piece::WHITE_KNIGHT).build(),
            //
            Move::builder(Square::D3, Square::F4, Piece::WHITE_KNIGHT).build(),
            Move::builder(Square::D3, Square::F2, Piece::WHITE_KNIGHT).build(),
            Move::builder(Square::D3, Square::C1, Piece::WHITE_KNIGHT).build(),
            //
            Move::builder(Square::C5, Square::A6, Piece::WHITE_KNIGHT)
                .captures(Piece::BLACK_BISHOP)
                .build(),
            Move::builder(Square::D3, Square::E5, Piece::WHITE_KNIGHT)
                .captures(Piece::BLACK_KNIGHT)
                .build(),
        ]
    }

    fn black_knight_moves() -> Vec<Move> {
        vec![
            Move::builder(Square::C6, Square::A7, Piece::BLACK_KNIGHT).build(),
            Move::builder(Square::C6, Square::B8, Piece::BLACK_KNIGHT).build(),
            Move::builder(Square::C6, Square::D4, Piece::BLACK_KNIGHT).build(),
            //
            Move::builder(Square::E5, Square::D7, Piece::BLACK_KNIGHT).build(),
            Move::builder(Square::E5, Square::F7, Piece::BLACK_KNIGHT).build(),
            Move::builder(Square::E5, Square::G6, Piece::BLACK_KNIGHT).build(),
            Move::builder(Square::E5, Square::F3, Piece::BLACK_KNIGHT).build(),
            Move::builder(Square::E5, Square::C4, Piece::BLACK_KNIGHT).build(),
            //
            Move::builder(Square::C6, Square::B4, Piece::BLACK_KNIGHT)
                .captures(Piece::WHITE_PAWN)
                .build(),
            Move::builder(Square::C6, Square::A5, Piece::BLACK_KNIGHT)
                .captures(Piece::WHITE_PAWN)
                .build(),
            //
            Move::builder(Square::E5, Square::G4, Piece::BLACK_KNIGHT)
                .captures(Piece::WHITE_QUEEN)
                .build(),
            Move::builder(Square::E5, Square::D3, Piece::BLACK_KNIGHT)
                .captures(Piece::WHITE_KNIGHT)
                .build(),
        ]
    }

    fn white_bishop_moves() -> Vec<Move> {
        vec![
            Move::builder(Square::B2, Square::A3, Piece::WHITE_BISHOP).build(),
            Move::builder(Square::B2, Square::C3, Piece::WHITE_BISHOP).build(),
            Move::builder(Square::B2, Square::D4, Piece::WHITE_BISHOP).build(),
            Move::builder(Square::B2, Square::C1, Piece::WHITE_BISHOP).build(),
            Move::builder(Square::B2, Square::A1, Piece::WHITE_BISHOP).build(),
            //
            Move::builder(Square::F1, Square::E2, Piece::WHITE_BISHOP).build(),
            Move::builder(Square::F1, Square::G2, Piece::WHITE_BISHOP).build(),
            Move::builder(Square::F1, Square::H3, Piece::WHITE_BISHOP).build(),
            //
            Move::builder(Square::B2, Square::E5, Piece::WHITE_BISHOP)
                .captures(Piece::BLACK_KNIGHT)
                .build(),
        ]
    }

    fn black_bishop_moves() -> Vec<Move> {
        vec![
            Move::builder(Square::A6, Square::B7, Piece::BLACK_BISHOP).build(),
            Move::builder(Square::A6, Square::C8, Piece::BLACK_BISHOP).build(),
            //
            Move::builder(Square::E3, Square::D4, Piece::BLACK_BISHOP).build(),
            Move::builder(Square::E3, Square::F4, Piece::BLACK_BISHOP).build(),
            Move::builder(Square::E3, Square::G5, Piece::BLACK_BISHOP).build(),
            Move::builder(Square::E3, Square::H6, Piece::BLACK_BISHOP).build(),
            Move::builder(Square::E3, Square::F2, Piece::BLACK_BISHOP).build(),
            Move::builder(Square::E3, Square::G1, Piece::BLACK_BISHOP).build(),
            //
            Move::builder(Square::E3, Square::C5, Piece::BLACK_BISHOP)
                .captures(Piece::WHITE_KNIGHT)
                .build(),
            Move::builder(Square::E3, Square::D2, Piece::BLACK_BISHOP)
                .captures(Piece::WHITE_PAWN)
                .build(),
        ]
    }

    fn white_rook_moves() -> Vec<Move> {
        vec![
            Move::builder(Square::H5, Square::G5, Piece::WHITE_ROOK).build(),
            Move::builder(Square::H5, Square::F5, Piece::WHITE_ROOK).build(),
            Move::builder(Square::H5, Square::H6, Piece::WHITE_ROOK).build(),
            Move::builder(Square::H5, Square::H7, Piece::WHITE_ROOK).build(),
            Move::builder(Square::H5, Square::H8, Piece::WHITE_ROOK).build(),
            Move::builder(Square::H5, Square::H4, Piece::WHITE_ROOK).build(),
            Move::builder(Square::H5, Square::H3, Piece::WHITE_ROOK).build(),
            //
            Move::builder(Square::H5, Square::E5, Piece::WHITE_ROOK)
                .captures(Piece::BLACK_KNIGHT)
                .build(),
        ]
    }

    fn black_rook_moves() -> Vec<Move> {
        vec![
            Move::builder(Square::F6, Square::E6, Piece::BLACK_ROOK).build(),
            Move::builder(Square::F6, Square::D6, Piece::BLACK_ROOK).build(),
            Move::builder(Square::F6, Square::G6, Piece::BLACK_ROOK).build(),
            Move::builder(Square::F6, Square::H6, Piece::BLACK_ROOK).build(),
            Move::builder(Square::F6, Square::F7, Piece::BLACK_ROOK).build(),
            Move::builder(Square::F6, Square::F8, Piece::BLACK_ROOK).build(),
            Move::builder(Square::F6, Square::F5, Piece::BLACK_ROOK).build(),
            Move::builder(Square::F6, Square::F4, Piece::BLACK_ROOK).build(),
            Move::builder(Square::F6, Square::F3, Piece::BLACK_ROOK).build(),
            Move::builder(Square::F6, Square::F2, Piece::BLACK_ROOK).build(),
            //
            Move::builder(Square::F6, Square::F1, Piece::BLACK_ROOK)
                .captures(Piece::WHITE_BISHOP)
                .build(),
        ]
    }

    fn white_queen_moves() -> Vec<Move> {
        vec![
            Move::builder(Square::G4, Square::F4, Piece::WHITE_QUEEN).build(),
            Move::builder(Square::G4, Square::E4, Piece::WHITE_QUEEN).build(),
            Move::builder(Square::G4, Square::D4, Piece::WHITE_QUEEN).build(),
            Move::builder(Square::G4, Square::C4, Piece::WHITE_QUEEN).build(),
            Move::builder(Square::G4, Square::F5, Piece::WHITE_QUEEN).build(),
            Move::builder(Square::G4, Square::E6, Piece::WHITE_QUEEN).build(),
            Move::builder(Square::G4, Square::D7, Piece::WHITE_QUEEN).build(),
            Move::builder(Square::G4, Square::C8, Piece::WHITE_QUEEN).build(),
            Move::builder(Square::G4, Square::G5, Piece::WHITE_QUEEN).build(),
            Move::builder(Square::G4, Square::G6, Piece::WHITE_QUEEN).build(),
            Move::builder(Square::G4, Square::G7, Piece::WHITE_QUEEN).build(),
            Move::builder(Square::G4, Square::G8, Piece::WHITE_QUEEN).build(),
            Move::builder(Square::G4, Square::H4, Piece::WHITE_QUEEN).build(),
            Move::builder(Square::G4, Square::H3, Piece::WHITE_QUEEN).build(),
            Move::builder(Square::G4, Square::F3, Piece::WHITE_QUEEN).build(),
            Move::builder(Square::G4, Square::E2, Piece::WHITE_QUEEN).build(),
            Move::builder(Square::G4, Square::D1, Piece::WHITE_QUEEN).build(),
            //
            Move::builder(Square::G4, Square::G3, Piece::WHITE_QUEEN)
                .captures(Piece::BLACK_PAWN)
                .build(),
        ]
    }

    fn black_queen_moves() -> Vec<Move> {
        vec![
            Move::builder(Square::B5, Square::A4, Piece::BLACK_QUEEN).build(),
            Move::builder(Square::B5, Square::B6, Piece::BLACK_QUEEN).build(),
            Move::builder(Square::B5, Square::B7, Piece::BLACK_QUEEN).build(),
            Move::builder(Square::B5, Square::B8, Piece::BLACK_QUEEN).build(),
            Move::builder(Square::B5, Square::C4, Piece::BLACK_QUEEN).build(),
            //
            Move::builder(Square::D8, Square::C8, Piece::BLACK_QUEEN).build(),
            Move::builder(Square::D8, Square::B8, Piece::BLACK_QUEEN).build(),
            Move::builder(Square::D8, Square::A8, Piece::BLACK_QUEEN).build(),
            Move::builder(Square::D8, Square::D7, Piece::BLACK_QUEEN).build(),
            Move::builder(Square::D8, Square::D6, Piece::BLACK_QUEEN).build(),
            Move::builder(Square::D8, Square::D5, Piece::BLACK_QUEEN).build(),
            Move::builder(Square::D8, Square::D4, Piece::BLACK_QUEEN).build(),
            Move::builder(Square::D8, Square::C7, Piece::BLACK_QUEEN).build(),
            Move::builder(Square::D8, Square::B6, Piece::BLACK_QUEEN).build(),
            //
            Move::builder(Square::B5, Square::A5, Piece::BLACK_QUEEN)
                .captures(Piece::WHITE_PAWN)
                .build(),
            Move::builder(Square::B5, Square::C5, Piece::BLACK_QUEEN)
                .captures(Piece::WHITE_KNIGHT)
                .build(),
            Move::builder(Square::B5, Square::D3, Piece::BLACK_QUEEN)
                .captures(Piece::WHITE_KNIGHT)
                .build(),
            Move::builder(Square::B5, Square::B4, Piece::BLACK_QUEEN)
                .captures(Piece::WHITE_PAWN)
                .build(),
            //
            Move::builder(Square::D8, Square::D3, Piece::BLACK_QUEEN)
                .captures(Piece::WHITE_KNIGHT)
                .build(),
            Move::builder(Square::D8, Square::A5, Piece::BLACK_QUEEN)
                .captures(Piece::WHITE_PAWN)
                .build(),
        ]
    }

    fn white_moves() -> Vec<Move> {
        vec![
            white_pawn_moves(),
            white_knight_moves(),
            white_bishop_moves(),
            white_rook_moves(),
            white_queen_moves(),
        ]
        .concat()
    }

    fn black_moves() -> Vec<Move> {
        vec![
            black_pawn_moves(),
            black_knight_moves(),
            black_bishop_moves(),
            black_rook_moves(),
            black_queen_moves(),
        ]
        .concat()
    }

    #[test]
    fn test_white_knight_moves() {
        let board = test_position();
        let mut moves = MoveVec::new();
        let mut move_gen = MoveGenerator::new(
            &board,
            Color::White,
            None,
            Default::default(),
            Default::default(),
            &mut moves,
        );

        move_gen.pseudo_legal_knight_moves(&Default::default());

        assert_in_any_order(moves, white_knight_moves());
    }

    #[test]
    fn test_black_knight_moves() {
        let board = test_position();
        let mut moves = MoveVec::new();
        let mut move_gen = MoveGenerator::new(
            &board,
            Color::Black,
            None,
            Default::default(),
            Default::default(),
            &mut moves,
        );

        move_gen.pseudo_legal_knight_moves(&Default::default());

        assert_in_any_order(moves, black_knight_moves());
    }

    #[test]
    fn test_white_bishop_moves() {
        let board = test_position();
        let mut moves = MoveVec::new();
        let mut move_gen = MoveGenerator::new(
            &board,
            Color::White,
            None,
            Default::default(),
            Default::default(),
            &mut moves,
        );

        move_gen.pseudo_legal_bishop_moves(&Default::default());

        assert_in_any_order(moves, white_bishop_moves());
    }

    #[test]
    fn test_black_bishop_moves() {
        let board = test_position();
        let mut moves = MoveVec::new();
        let mut move_gen = MoveGenerator::new(
            &board,
            Color::Black,
            None,
            Default::default(),
            Default::default(),
            &mut moves,
        );

        move_gen.pseudo_legal_bishop_moves(&Default::default());

        assert_in_any_order(moves, black_bishop_moves());
    }

    #[test]
    fn test_white_rook_moves() {
        let board = test_position();
        let mut moves = MoveVec::new();
        let mut move_gen = MoveGenerator::new(
            &board,
            Color::White,
            None,
            Default::default(),
            Default::default(),
            &mut moves,
        );

        move_gen.pseudo_legal_rook_moves(&Default::default());

        assert_in_any_order(moves, white_rook_moves());
    }

    #[test]
    fn test_black_rook_moves() {
        let board = test_position();
        let mut moves = MoveVec::new();
        let mut move_gen = MoveGenerator::new(
            &board,
            Color::Black,
            None,
            Default::default(),
            Default::default(),
            &mut moves,
        );

        move_gen.pseudo_legal_rook_moves(&Default::default());

        assert_in_any_order(moves, black_rook_moves());
    }

    #[test]
    fn test_white_queen_moves() {
        let board = test_position();
        let mut moves = MoveVec::new();
        let mut move_gen = MoveGenerator::new(
            &board,
            Color::White,
            None,
            Default::default(),
            Default::default(),
            &mut moves,
        );

        move_gen.pseudo_legal_queen_moves(&Default::default());

        assert_in_any_order(moves, white_queen_moves());
    }

    #[test]
    fn test_black_queen_moves() {
        let board = test_position();
        let mut moves = MoveVec::new();
        let mut move_gen = MoveGenerator::new(
            &board,
            Color::Black,
            None,
            Default::default(),
            Default::default(),
            &mut moves,
        );

        move_gen.pseudo_legal_queen_moves(&Default::default());

        assert_in_any_order(moves, black_queen_moves());
    }

    #[test]
    fn test_white_moves() {
        let board = test_position();
        let mut moves = MoveVec::new();
        let mut move_gen = MoveGenerator::new(
            &board,
            Color::White,
            None,
            Default::default(),
            Default::default(),
            &mut moves,
        );

        move_gen.pseudo_legal_moves(&Default::default());

        assert_in_any_order(moves, white_moves());
    }

    #[test]
    fn test_black_moves() {
        let board = test_position();
        let mut moves = MoveVec::new();
        let mut move_gen = MoveGenerator::new(
            &board,
            Color::Black,
            None,
            Default::default(),
            Default::default(),
            &mut moves,
        );

        move_gen.pseudo_legal_moves(&Default::default());

        assert_in_any_order(moves, black_moves());
    }

    #[test]
    fn test_push_mask() {
        let board = test_position();
        let mut moves = MoveVec::new();
        let mut move_gen = MoveGenerator::new(
            &board,
            Color::White,
            None,
            Default::default(),
            Default::default(),
            &mut moves,
        );

        move_gen.pseudo_legal_moves(&MoveGenMasks {
            capture: Bitboard::EMPTY,
            push: Bitboard::from_square(Square::H4),
            movable: Bitboard::ALL,
        });

        assert_in_any_order(
            moves,
            white_moves()
                .into_iter()
                .filter(|m| m.to_square() == Square::H4),
        );
    }

    #[test]
    fn test_capture_mask() {
        let board = test_position();
        let mut moves = MoveVec::new();
        let mut move_gen = MoveGenerator::new(
            &board,
            Color::White,
            None,
            Default::default(),
            Default::default(),
            &mut moves,
        );

        move_gen.pseudo_legal_moves(&MoveGenMasks {
            capture: Bitboard::from_square(Square::E5),
            push: Bitboard::EMPTY,
            movable: Bitboard::ALL,
        });

        assert_in_any_order(
            moves,
            white_moves()
                .into_iter()
                .filter(|m| m.to_square() == Square::E5),
        );
    }

    #[test]
    fn test_movable_mask() {
        let board = test_position();
        let mut moves = MoveVec::new();
        let mut move_gen = MoveGenerator::new(
            &board,
            Color::White,
            None,
            Default::default(),
            Default::default(),
            &mut moves,
        );

        move_gen.pseudo_legal_moves(&MoveGenMasks {
            capture: Bitboard::ALL,
            push: Bitboard::ALL,
            movable: Bitboard::from_square(Square::G4),
        });

        assert_in_any_order(moves, white_queen_moves());
    }
}
