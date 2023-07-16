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

    fn pseudo_legal_en_passants(&mut self, masks: &MoveGenMasks) {
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

pub fn white_pawn_attacks(white_pawns: Bitboard) -> Bitboard {
    white_pawns.step_north_east() | white_pawns.step_north_west()
}

pub fn black_pawn_attacks(black_pawns: Bitboard) -> Bitboard {
    black_pawns.step_south_east() | black_pawns.step_south_west()
}
