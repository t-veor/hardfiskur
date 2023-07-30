use crate::{
    board::{Bitboard, Color, Move, PieceType, Square},
    move_gen::{MoveGenMasks, MoveGenerator, POSSIBLE_PROMOTIONS},
};

impl<'board, 'moves> MoveGenerator<'board, 'moves> {
    pub(in crate::move_gen) fn pseudo_legal_pawn_moves(&mut self, masks: &MoveGenMasks) {
        self.pseudo_legal_pawn_pushes(masks);
        self.pseudo_legal_pawn_captures(masks);
        self.pseudo_legal_en_passants(masks);
    }

    pub(in crate::move_gen) fn pseudo_legal_pawn_pushes(&mut self, masks: &MoveGenMasks) {
        let piece = PieceType::Pawn.with_color(self.to_move);
        let movable_pawns = self.board[piece] & masks.movable;

        let (single_pushable_pawns, double_pushable_pawns) = match self.to_move {
            Color::White => (
                white_pawns_able_to_push(movable_pawns, self.empty & masks.push),
                white_pawns_able_to_double_push(movable_pawns, self.empty, masks.push),
            ),
            Color::Black => (
                black_pawns_able_to_push(movable_pawns, self.empty & masks.push),
                black_pawns_able_to_double_push(movable_pawns, self.empty, masks.push),
            ),
        };

        let rank_before_promotion = if self.to_move.is_white() { 6 } else { 1 };

        for from in single_pushable_pawns.squares() {
            let to = pawn_push_dest(from, self.to_move);

            if from.rank() == rank_before_promotion {
                for &promo in POSSIBLE_PROMOTIONS {
                    self.out_moves
                        .push(Move::builder(from, to, piece).promotes_to(promo).build());
                }
            } else {
                self.out_moves.push(Move::builder(from, to, piece).build());
            }
        }

        for from in double_pushable_pawns.squares() {
            let to = pawn_double_push_dest(from, self.to_move);

            self.out_moves
                .push(Move::builder(from, to, piece).is_double_pawn_push().build());
        }
    }

    pub(in crate::move_gen) fn pseudo_legal_pawn_captures(&mut self, masks: &MoveGenMasks) {
        let piece = PieceType::Pawn.with_color(self.to_move);
        let movable_pawns = self.board[piece] & masks.movable;
        let capturable_pieces = self.board[self.to_move.flip()] & masks.capture;

        let (east_captures, west_captures) = match self.to_move {
            Color::White => (
                white_pawns_able_to_capture_east(movable_pawns, capturable_pieces),
                white_pawns_able_to_capture_west(movable_pawns, capturable_pieces),
            ),
            Color::Black => (
                black_pawns_able_to_capture_east(movable_pawns, capturable_pieces),
                black_pawns_able_to_capture_west(movable_pawns, capturable_pieces),
            ),
        };

        let rank_before_promotion = if self.to_move.is_white() { 6 } else { 1 };

        let mut push_capture = |from: Square, to: Square| {
            let captured_piece = self
                .board
                .piece_with_color_at(self.to_move.flip(), to)
                .unwrap();

            if from.rank() == rank_before_promotion {
                for &promo in POSSIBLE_PROMOTIONS {
                    self.out_moves.push(
                        Move::builder(from, to, piece)
                            .captures(captured_piece)
                            .promotes_to(promo)
                            .build(),
                    );
                }
            } else {
                self.out_moves.push(
                    Move::builder(from, to, piece)
                        .captures(captured_piece)
                        .build(),
                );
            }
        };

        for from in east_captures.squares() {
            push_capture(from, pawn_east_capture_dest(from, self.to_move));
        }

        for from in west_captures.squares() {
            push_capture(from, pawn_west_capture_dest(from, self.to_move));
        }
    }

    pub(in crate::move_gen) fn pseudo_legal_en_passants(&mut self, masks: &MoveGenMasks) {
        let en_passant = match self.en_passant {
            Some(en_passant) => en_passant,
            None => return,
        };

        let kings = self.board[PieceType::King.with_color(self.to_move)];
        let king_square = kings.to_square().unwrap();

        let pawn = PieceType::Pawn.with_color(self.to_move);
        let opponent_pawn = PieceType::Pawn.with_color(self.to_move.flip());

        let movable_pawns = self.board[pawn] & masks.movable;
        let opponent_pawns = self.board[opponent_pawn];

        let en_passant_bb = Bitboard::from_square(en_passant);

        let (east_captures, west_captures) = match self.to_move {
            Color::White => (
                white_pawns_able_to_capture_east(movable_pawns, en_passant_bb),
                white_pawns_able_to_capture_west(movable_pawns, en_passant_bb),
            ),
            Color::Black => (
                black_pawns_able_to_capture_east(movable_pawns, en_passant_bb),
                black_pawns_able_to_capture_west(movable_pawns, en_passant_bb),
            ),
        };

        let mut try_en_passant = |from: Square, to: Square| {
            let captured_pawn_square = Square::new_unchecked(from.rank(), to.file());

            // Will this en passant push the pawn into a position to block a
            // check, if there is one?
            if (en_passant_bb & masks.push).has_piece() {
                // Fine -- either there is no check or the pawn is pushed into a
                // position to block the check
            } else {
                // Otherwise, will this en passant capture a double-moved pawn
                // that's giving check, if there is one?
                if !(opponent_pawns & masks.capture).get(captured_pawn_square) {
                    // En passant not possible due to pins
                    return;
                }
            }

            // Does removing the en passant pawn and the captured pawn reveal
            // a horizontal attack on the king?
            let mut occupied_without_en_passant_pawns = self.occupied;
            occupied_without_en_passant_pawns.reset(from);
            occupied_without_en_passant_pawns.reset(captured_pawn_square);

            let opponent_rooks = self.board[PieceType::Rook.with_color(self.to_move.flip())];
            let opponent_queens = self.board[PieceType::Queen.with_color(self.to_move.flip())];

            let potential_horizontal_attackers = self
                .lookups
                .get_rook_attacks(occupied_without_en_passant_pawns, king_square)
                & Bitboard::rank_mask(king_square.rank());

            if (potential_horizontal_attackers & (opponent_rooks | opponent_queens)).has_piece() {
                return;
            }

            // Getting here means no pin issues from the en passant
            self.out_moves.push(
                Move::builder(from, to, PieceType::Pawn.with_color(self.to_move))
                    .captures(opponent_pawn)
                    .is_en_passant()
                    .build(),
            )
        };

        for from in east_captures.squares() {
            try_en_passant(from, en_passant);
        }

        for from in west_captures.squares() {
            try_en_passant(from, en_passant);
        }
    }
}

fn pawn_push_dest(square: Square, color: Color) -> Square {
    square.offset(match color {
        Color::White => 8,
        Color::Black => -8,
    })
}

fn pawn_double_push_dest(square: Square, color: Color) -> Square {
    square.offset(match color {
        Color::White => 16,
        Color::Black => -16,
    })
}

fn pawn_east_capture_dest(square: Square, color: Color) -> Square {
    square.offset(match color {
        Color::White => 9,
        Color::Black => -7,
    })
}

fn pawn_west_capture_dest(square: Square, color: Color) -> Square {
    square.offset(match color {
        Color::White => 7,
        Color::Black => -9,
    })
}

fn white_pawns_able_to_push(movable_white_pawns: Bitboard, can_push_into: Bitboard) -> Bitboard {
    can_push_into.step_south() & movable_white_pawns
}

fn white_pawns_able_to_double_push(
    movable_white_pawns: Bitboard,
    empty: Bitboard,
    push_mask: Bitboard,
) -> Bitboard {
    let can_push_into_on_rank_3 = (Bitboard::RANK_4 & empty & push_mask).step_south() & empty;
    white_pawns_able_to_push(movable_white_pawns, can_push_into_on_rank_3)
}

fn white_pawns_able_to_capture_east(
    movable_white_pawns: Bitboard,
    capturable_pieces: Bitboard,
) -> Bitboard {
    movable_white_pawns & capturable_pieces.step_south_west()
}

fn white_pawns_able_to_capture_west(
    movable_white_pawns: Bitboard,
    capturable_pieces: Bitboard,
) -> Bitboard {
    movable_white_pawns & capturable_pieces.step_south_east()
}

fn black_pawns_able_to_push(movable_black_pawns: Bitboard, can_push_into: Bitboard) -> Bitboard {
    can_push_into.step_north() & movable_black_pawns
}

fn black_pawns_able_to_double_push(
    movable_black_pawns: Bitboard,
    empty: Bitboard,
    push_mask: Bitboard,
) -> Bitboard {
    let can_push_into_on_rank_6 = (Bitboard::RANK_5 & empty & push_mask).step_north() & empty;
    black_pawns_able_to_push(movable_black_pawns, can_push_into_on_rank_6)
}

fn black_pawns_able_to_capture_east(
    movable_black_pawns: Bitboard,
    capturable_pieces: Bitboard,
) -> Bitboard {
    movable_black_pawns & capturable_pieces.step_north_west()
}

fn black_pawns_able_to_capture_west(
    movable_black_pawns: Bitboard,
    capturable_pieces: Bitboard,
) -> Bitboard {
    movable_black_pawns & capturable_pieces.step_north_east()
}

/// Calculate squares attacked by white pawns.
pub fn white_pawn_attacks(white_pawns: Bitboard) -> Bitboard {
    white_pawns.step_north_east() | white_pawns.step_north_west()
}

/// Calculate squares attacked by black pawns.
pub fn black_pawn_attacks(black_pawns: Bitboard) -> Bitboard {
    black_pawns.step_south_east() | black_pawns.step_south_west()
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use crate::{
        board::{BoardRepr, Piece},
        move_gen::MoveVec,
        test_utils::assert_in_any_order,
    };

    use super::*;

    fn call_and_get_moves(
        board: &BoardRepr,
        color: Color,
        f: impl FnOnce(&mut MoveGenerator),
    ) -> MoveVec {
        let mut moves = MoveVec::new();
        let mut move_gen = MoveGenerator::new(
            &board,
            color,
            None,
            Default::default(),
            Default::default(),
            &mut moves,
        );
        f(&mut move_gen);

        moves
    }

    fn pawn_test_board() -> BoardRepr {
        "
            r...k.n.
            pPp...Pp
            .N.....B
            .NbpPp..
            QpPP....
            ......qP
            P..p.PpP
            .K.R.B.R
        "
        .parse()
        .unwrap()
    }

    fn expected_white_pawn_pushes() -> Vec<Move> {
        vec![
            Move::builder(Square::A2, Square::A3, Piece::WHITE_PAWN).build(),
            Move::builder(Square::B7, Square::B8, Piece::WHITE_PAWN)
                .promotes_to(PieceType::Queen)
                .build(),
            Move::builder(Square::B7, Square::B8, Piece::WHITE_PAWN)
                .promotes_to(PieceType::Knight)
                .build(),
            Move::builder(Square::B7, Square::B8, Piece::WHITE_PAWN)
                .promotes_to(PieceType::Rook)
                .build(),
            Move::builder(Square::B7, Square::B8, Piece::WHITE_PAWN)
                .promotes_to(PieceType::Bishop)
                .build(),
            Move::builder(Square::E5, Square::E6, Piece::WHITE_PAWN).build(),
            Move::builder(Square::F2, Square::F3, Piece::WHITE_PAWN).build(),
            Move::builder(Square::F2, Square::F4, Piece::WHITE_PAWN)
                .is_double_pawn_push()
                .build(),
            Move::builder(Square::H3, Square::H4, Piece::WHITE_PAWN).build(),
        ]
    }

    fn expected_white_pawn_captures() -> Vec<Move> {
        vec![
            Move::builder(Square::B7, Square::A8, Piece::WHITE_PAWN)
                .captures(Piece::BLACK_ROOK)
                .promotes_to(PieceType::Queen)
                .build(),
            Move::builder(Square::B7, Square::A8, Piece::WHITE_PAWN)
                .captures(Piece::BLACK_ROOK)
                .promotes_to(PieceType::Knight)
                .build(),
            Move::builder(Square::B7, Square::A8, Piece::WHITE_PAWN)
                .captures(Piece::BLACK_ROOK)
                .promotes_to(PieceType::Rook)
                .build(),
            Move::builder(Square::B7, Square::A8, Piece::WHITE_PAWN)
                .captures(Piece::BLACK_ROOK)
                .promotes_to(PieceType::Bishop)
                .build(),
            Move::builder(Square::C4, Square::D5, Piece::WHITE_PAWN)
                .captures(Piece::BLACK_PAWN)
                .build(),
            Move::builder(Square::D4, Square::C5, Piece::WHITE_PAWN)
                .captures(Piece::BLACK_BISHOP)
                .build(),
            Move::builder(Square::F2, Square::G3, Piece::WHITE_PAWN)
                .captures(Piece::BLACK_QUEEN)
                .build(),
            Move::builder(Square::H2, Square::G3, Piece::WHITE_PAWN)
                .captures(Piece::BLACK_QUEEN)
                .build(),
        ]
    }

    fn expected_white_pawn_en_passants() -> Vec<Move> {
        vec![Move::builder(Square::E5, Square::D6, Piece::WHITE_PAWN)
            .captures(Piece::BLACK_PAWN)
            .is_en_passant()
            .build()]
    }

    fn expected_white_pawn_moves() -> Vec<Move> {
        vec![
            expected_white_pawn_pushes(),
            expected_white_pawn_captures(),
            expected_white_pawn_en_passants(),
        ]
        .concat()
    }

    fn expected_black_pawn_pushes() -> Vec<Move> {
        vec![
            Move::builder(Square::A7, Square::A6, Piece::BLACK_PAWN).build(),
            Move::builder(Square::A7, Square::A5, Piece::BLACK_PAWN)
                .is_double_pawn_push()
                .build(),
            Move::builder(Square::B4, Square::B3, Piece::BLACK_PAWN).build(),
            Move::builder(Square::C7, Square::C6, Piece::BLACK_PAWN).build(),
            Move::builder(Square::F5, Square::F4, Piece::BLACK_PAWN).build(),
            Move::builder(Square::G2, Square::G1, Piece::BLACK_PAWN)
                .promotes_to(PieceType::Queen)
                .build(),
            Move::builder(Square::G2, Square::G1, Piece::BLACK_PAWN)
                .promotes_to(PieceType::Knight)
                .build(),
            Move::builder(Square::G2, Square::G1, Piece::BLACK_PAWN)
                .promotes_to(PieceType::Rook)
                .build(),
            Move::builder(Square::G2, Square::G1, Piece::BLACK_PAWN)
                .promotes_to(PieceType::Bishop)
                .build(),
        ]
    }

    fn expected_black_pawn_captures() -> Vec<Move> {
        vec![
            Move::builder(Square::A7, Square::B6, Piece::BLACK_PAWN)
                .captures(Piece::WHITE_KNIGHT)
                .build(),
            Move::builder(Square::C7, Square::B6, Piece::BLACK_PAWN)
                .captures(Piece::WHITE_KNIGHT)
                .build(),
            Move::builder(Square::D5, Square::C4, Piece::BLACK_PAWN)
                .captures(Piece::WHITE_PAWN)
                .build(),
            Move::builder(Square::G2, Square::F1, Piece::BLACK_PAWN)
                .captures(Piece::WHITE_BISHOP)
                .promotes_to(PieceType::Queen)
                .build(),
            Move::builder(Square::G2, Square::F1, Piece::BLACK_PAWN)
                .captures(Piece::WHITE_BISHOP)
                .promotes_to(PieceType::Knight)
                .build(),
            Move::builder(Square::G2, Square::F1, Piece::BLACK_PAWN)
                .captures(Piece::WHITE_BISHOP)
                .promotes_to(PieceType::Rook)
                .build(),
            Move::builder(Square::G2, Square::F1, Piece::BLACK_PAWN)
                .captures(Piece::WHITE_BISHOP)
                .promotes_to(PieceType::Bishop)
                .build(),
            Move::builder(Square::G2, Square::H1, Piece::BLACK_PAWN)
                .captures(Piece::WHITE_ROOK)
                .promotes_to(PieceType::Queen)
                .build(),
            Move::builder(Square::G2, Square::H1, Piece::BLACK_PAWN)
                .captures(Piece::WHITE_ROOK)
                .promotes_to(PieceType::Knight)
                .build(),
            Move::builder(Square::G2, Square::H1, Piece::BLACK_PAWN)
                .captures(Piece::WHITE_ROOK)
                .promotes_to(PieceType::Rook)
                .build(),
            Move::builder(Square::G2, Square::H1, Piece::BLACK_PAWN)
                .captures(Piece::WHITE_ROOK)
                .promotes_to(PieceType::Bishop)
                .build(),
        ]
    }

    fn expected_black_pawn_en_passants() -> Vec<Move> {
        vec![Move::builder(Square::B4, Square::C3, Piece::BLACK_PAWN)
            .captures(Piece::WHITE_PAWN)
            .is_en_passant()
            .build()]
    }

    fn expected_black_pawn_moves() -> Vec<Move> {
        vec![
            expected_black_pawn_pushes(),
            expected_black_pawn_captures(),
            expected_black_pawn_en_passants(),
        ]
        .concat()
    }

    #[test]
    fn white_pawn_pushes() {
        let board = pawn_test_board();

        let moves = call_and_get_moves(&board, Color::White, |move_gen| {
            move_gen.pseudo_legal_pawn_pushes(&Default::default());
        });

        assert_in_any_order(moves, expected_white_pawn_pushes());
    }

    #[test]
    fn white_pawn_captures() {
        let board = pawn_test_board();

        let moves = call_and_get_moves(&board, Color::White, |move_gen| {
            move_gen.pseudo_legal_pawn_captures(&Default::default());
        });

        assert_in_any_order(moves, expected_white_pawn_captures());
    }

    #[test]
    fn white_pawn_en_passants() {
        let board = pawn_test_board();
        let mut moves = MoveVec::new();
        let mut move_gen = MoveGenerator::new(
            &board,
            Color::White,
            Some(Square::D6),
            Default::default(),
            Default::default(),
            &mut moves,
        );

        move_gen.pseudo_legal_en_passants(&Default::default());

        assert_in_any_order(moves, expected_white_pawn_en_passants());
    }

    #[test]
    fn white_pawn_moves() {
        let board = pawn_test_board();
        let mut moves = MoveVec::new();
        let mut move_gen = MoveGenerator::new(
            &board,
            Color::White,
            Some(Square::D6),
            Default::default(),
            Default::default(),
            &mut moves,
        );

        move_gen.pseudo_legal_pawn_moves(&Default::default());

        assert_in_any_order(moves, expected_white_pawn_moves());
    }

    #[test]
    fn white_pawn_push_mask() {
        let board = pawn_test_board();

        let moves = call_and_get_moves(&board, Color::White, |move_gen| {
            move_gen.pseudo_legal_pawn_moves(&MoveGenMasks {
                capture: Bitboard::EMPTY,
                push: Bitboard::from_square(Square::F4) | Bitboard::from_square(Square::H4),
                movable: Bitboard::ALL,
            });
        });

        assert_in_any_order(
            moves,
            expected_white_pawn_moves()
                .into_iter()
                .filter(|m| m.to_square() == Square::F4 || m.to_square() == Square::H4),
        )
    }

    #[test]
    fn white_pawn_capture_mask() {
        let board = pawn_test_board();

        let moves = call_and_get_moves(&board, Color::White, |move_gen| {
            move_gen.pseudo_legal_pawn_moves(&MoveGenMasks {
                capture: Bitboard::from_square(Square::A8) | Bitboard::from_square(Square::G3),
                push: Bitboard::EMPTY,
                movable: Bitboard::ALL,
            });
        });

        assert_in_any_order(
            moves,
            expected_white_pawn_moves()
                .into_iter()
                .filter(|m| m.to_square() == Square::A8 || m.to_square() == Square::G3),
        )
    }

    #[test]
    fn white_pawn_movable_mask() {
        let board = pawn_test_board();

        let moves = call_and_get_moves(&board, Color::White, |move_gen| {
            move_gen.pseudo_legal_pawn_moves(&MoveGenMasks {
                capture: Bitboard::ALL,
                push: Bitboard::ALL,
                movable: Bitboard::from_square(Square::F3),
            });
        });

        assert_in_any_order(
            moves,
            expected_white_pawn_moves()
                .into_iter()
                .filter(|m| m.from_square() == Square::F3),
        )
    }

    #[test]
    fn black_pawn_pushes() {
        let board = pawn_test_board();

        let moves = call_and_get_moves(&board, Color::Black, |move_gen| {
            move_gen.pseudo_legal_pawn_pushes(&Default::default());
        });

        assert_in_any_order(moves, expected_black_pawn_pushes());
    }

    #[test]
    fn black_pawn_captures() {
        let board = pawn_test_board();

        let moves = call_and_get_moves(&board, Color::Black, |move_gen| {
            move_gen.pseudo_legal_pawn_captures(&Default::default());
        });

        assert_in_any_order(moves, expected_black_pawn_captures());
    }

    #[test]
    fn black_pawn_en_passants() {
        let board = pawn_test_board();
        let mut moves = MoveVec::new();
        let mut move_gen = MoveGenerator::new(
            &board,
            Color::Black,
            Some(Square::C3),
            Default::default(),
            Default::default(),
            &mut moves,
        );

        move_gen.pseudo_legal_en_passants(&Default::default());

        assert_in_any_order(moves, expected_black_pawn_en_passants());
    }

    #[test]
    fn black_pawn_moves() {
        let board = pawn_test_board();
        let mut moves = MoveVec::new();
        let mut move_gen = MoveGenerator::new(
            &board,
            Color::Black,
            Some(Square::C3),
            Default::default(),
            Default::default(),
            &mut moves,
        );

        move_gen.pseudo_legal_pawn_moves(&Default::default());

        assert_in_any_order(moves, expected_black_pawn_moves());
    }

    #[test]
    fn black_pawn_push_mask() {
        let board = pawn_test_board();

        let moves = call_and_get_moves(&board, Color::Black, |move_gen| {
            move_gen.pseudo_legal_pawn_moves(&MoveGenMasks {
                capture: Bitboard::EMPTY,
                push: Bitboard::from_square(Square::A5) | Bitboard::from_square(Square::C6),
                movable: Bitboard::ALL,
            });
        });

        assert_in_any_order(
            moves,
            expected_black_pawn_moves()
                .into_iter()
                .filter(|m| m.to_square() == Square::A5 || m.to_square() == Square::C6),
        )
    }

    #[test]
    fn black_pawn_capture_mask() {
        let board = pawn_test_board();

        let moves = call_and_get_moves(&board, Color::Black, |move_gen| {
            move_gen.pseudo_legal_pawn_moves(&MoveGenMasks {
                capture: Bitboard::from_square(Square::F1),
                push: Bitboard::EMPTY,
                movable: Bitboard::ALL,
            });
        });

        assert_in_any_order(
            moves,
            expected_black_pawn_moves()
                .into_iter()
                .filter(|m| m.to_square() == Square::F1),
        )
    }

    #[test]
    fn black_pawn_movable_mask() {
        let board = pawn_test_board();

        let moves = call_and_get_moves(&board, Color::Black, |move_gen| {
            move_gen.pseudo_legal_pawn_moves(&MoveGenMasks {
                capture: Bitboard::ALL,
                push: Bitboard::ALL,
                movable: Bitboard::from_square(Square::G2),
            });
        });

        assert_in_any_order(
            moves,
            expected_black_pawn_moves()
                .into_iter()
                .filter(|m| m.from_square() == Square::G2),
        )
    }

    #[test]
    fn en_passant_blocks_check() {
        let board = "
            ...b..k.
            ........
            ........
            ....PpK.
            ........
            ........
            ........
            ........
        "
        .parse()
        .unwrap();

        let mut moves = MoveVec::new();
        let mut move_gen = MoveGenerator::new(
            &board,
            Color::White,
            Some(Square::F6),
            Default::default(),
            Default::default(),
            &mut moves,
        );

        move_gen.pseudo_legal_en_passants(&MoveGenMasks {
            capture: Bitboard::from_square(Square::D8),
            push: Bitboard::from_square(Square::E7) | Bitboard::from_square(Square::F6),
            movable: Bitboard::ALL,
        });

        assert_in_any_order(
            moves,
            vec![Move::builder(Square::E5, Square::F6, Piece::WHITE_PAWN)
                .captures(Piece::BLACK_PAWN)
                .is_en_passant()
                .build()],
        );
    }

    #[test]
    fn en_passant_captures_attacking_pawn() {
        let board = "
            ......k.
            ........
            ........
            ....Pp..
            ....K...
            ........
            ........
            ........
        "
        .parse()
        .unwrap();

        let mut moves = MoveVec::new();
        let mut move_gen = MoveGenerator::new(
            &board,
            Color::White,
            Some(Square::F6),
            Default::default(),
            Default::default(),
            &mut moves,
        );

        move_gen.pseudo_legal_en_passants(&MoveGenMasks {
            capture: Bitboard::from_square(Square::F5),
            push: Bitboard::EMPTY,
            movable: Bitboard::ALL,
        });

        assert_in_any_order(
            moves,
            vec![Move::builder(Square::E5, Square::F6, Piece::WHITE_PAWN)
                .captures(Piece::BLACK_PAWN)
                .is_en_passant()
                .build()],
        );
    }

    #[test]
    fn en_passant_not_possible_in_discovered_check() {
        let board = "
            ..k.....
            ....r..K
            ........
            ....Pp..
            ........
            ........
            ........
            ........
        "
        .parse()
        .unwrap();

        let mut moves = MoveVec::new();
        let mut move_gen = MoveGenerator::new(
            &board,
            Color::White,
            Some(Square::F6),
            Default::default(),
            Default::default(),
            &mut moves,
        );

        move_gen.pseudo_legal_en_passants(&MoveGenMasks {
            capture: Bitboard::from_square(Square::E7),
            push: Bitboard::from_square(Square::F7) | Bitboard::from_square(Square::G7),
            movable: Bitboard::ALL,
        });

        assert!(moves.is_empty());
    }

    #[test]
    fn en_passant_not_possible_due_to_double_pin() {
        let board = "
            ......k.
            ........
            ........
            .r..Pp.K
            ........
            ........
            ........
            ........
        "
        .parse()
        .unwrap();

        let mut moves = MoveVec::new();
        let mut move_gen = MoveGenerator::new(
            &board,
            Color::White,
            Some(Square::F6),
            Default::default(),
            Default::default(),
            &mut moves,
        );

        move_gen.pseudo_legal_en_passants(&Default::default());

        assert!(moves.is_empty());
    }

    #[test]
    fn en_passant_multiple_options() {
        let board = "
            ......k.
            ........
            ........
            ....PpP.
            ........
            .....K..
            ........
            ........
        "
        .parse()
        .unwrap();

        let mut moves = MoveVec::new();
        let mut move_gen = MoveGenerator::new(
            &board,
            Color::White,
            Some(Square::F6),
            Default::default(),
            Default::default(),
            &mut moves,
        );

        move_gen.pseudo_legal_en_passants(&Default::default());

        assert_in_any_order(
            moves,
            vec![
                Move::builder(Square::E5, Square::F6, Piece::WHITE_PAWN)
                    .captures(Piece::BLACK_PAWN)
                    .is_en_passant()
                    .build(),
                Move::builder(Square::G5, Square::F6, Piece::WHITE_PAWN)
                    .captures(Piece::BLACK_PAWN)
                    .is_en_passant()
                    .build(),
            ],
        );
    }

    #[test]
    fn test_white_pawn_attacks() {
        let board = pawn_test_board();
        assert_eq!(
            white_pawn_attacks(board[Piece::WHITE_PAWN]),
            "
                #.#..#.#
                ........
                ...#.#..
                .####...
                ......#.
                .#..#.#.
                ........
                ........
        "
            .parse()
            .unwrap()
        )
    }

    #[test]
    fn test_black_pawn_attacks() {
        let board = pawn_test_board();
        assert_eq!(
            black_pawn_attacks(board[Piece::BLACK_PAWN]),
            "
                ........
                ........
                .#.#..#.
                ........
                ..#.#.#.
                #.#.....
                ........
                ..#.##.#
        "
            .parse()
            .unwrap()
        )
    }
}
