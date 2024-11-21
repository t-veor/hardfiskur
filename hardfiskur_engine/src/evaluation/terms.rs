use hardfiskur_core::board::{Bitboard, Color, Piece, PieceType, Square};

use crate::evaluation::parameters::{
    BISHOP_MOBILITY, KNIGHT_MOBILITY, PIECE_SQUARE_TABLES, QUEEN_MOBILITY, ROOK_MOBILITY,
};

use super::{
    lookups::{
        PASSED_PAWN_MASKS, PAWN_SHIELD_CLOSE_MASKS, PAWN_SHIELD_FAR_MASKS, SENSIBLE_KING_MASKS,
    },
    packed_score::S,
    parameters::{
        DOUBLED_PAWNS, ISOLATED_PAWNS, MATERIAL, PASSED_PAWNS, PAWN_SHIELD_CLOSE, PAWN_SHIELD_FAR,
        PAWN_STORM,
    },
    template_params::{ColorParam, PieceTypeParam},
    trace::Trace,
    EvalContext,
};

impl<'a> EvalContext<'a> {
    #[inline]
    pub fn material<C: ColorParam>(&self, piece_type: PieceType, trace: &mut impl Trace) -> S {
        trace.add(|t| t.material[piece_type.index()] += C::COEFF);

        C::SIGN * MATERIAL[piece_type.index()]
    }

    #[inline]
    pub fn piece_square_table<C: ColorParam>(
        &self,
        piece_type: PieceType,
        square: Square,
        trace: &mut impl Trace,
    ) -> S {
        let square = if C::IS_WHITE { square.flip() } else { square };

        trace.add(|t| {
            let table = match piece_type {
                PieceType::Pawn => &mut t.pawn_pst,
                PieceType::Knight => &mut t.knight_pst,
                PieceType::Bishop => &mut t.bishop_pst,
                PieceType::Rook => &mut t.rook_pst,
                PieceType::Queen => &mut t.queen_pst,
                PieceType::King => &mut t.king_pst,
            };

            table[square.index()] += C::COEFF;
        });

        C::SIGN * PIECE_SQUARE_TABLES[piece_type.index()][square.index()]
    }

    #[inline]
    pub fn mobility<C: ColorParam, P: PieceTypeParam>(&self, trace: &mut impl Trace) -> S {
        const {
            assert!(
                !matches!(P::PIECE_TYPE, PieceType::Pawn | PieceType::King),
                "Can't call mobility() with Pawn or King!"
            );
        }

        let mut total = S::ZERO;

        let mobility_squares = if C::IS_WHITE {
            !self.pawns.pawn_attacks[Color::Black.index()]
        } else {
            !self.pawns.pawn_attacks[Color::White.index()]
        } & !self.board.get_bitboard_for_color(C::COLOR);

        let piece_bb = self
            .board
            .get_bitboard_for_piece(Piece::new(C::COLOR, P::PIECE_TYPE));

        for square in piece_bb.squares() {
            let mobility_bb = match P::PIECE_TYPE {
                PieceType::Knight => self.lookups.get_knight_moves(square),
                PieceType::Bishop => self.lookups.get_bishop_attacks(self.occupied, square),
                PieceType::Rook => self.lookups.get_rook_attacks(self.occupied, square),
                PieceType::Queen => self.lookups.get_queen_attacks(self.occupied, square),
                PieceType::Pawn | PieceType::King => unreachable!(),
            } & mobility_squares;

            let mobility_count = mobility_bb.pop_count() as usize;

            trace.add(|t| match P::PIECE_TYPE {
                PieceType::Knight => t.knight_mobility[mobility_count] += C::COEFF,
                PieceType::Bishop => t.bishop_mobility[mobility_count] += C::COEFF,
                PieceType::Rook => t.rook_mobility[mobility_count] += C::COEFF,
                PieceType::Queen => t.queen_mobility[mobility_count] += C::COEFF,
                PieceType::Pawn | PieceType::King => unreachable!(),
            });

            total += C::SIGN
                * match P::PIECE_TYPE {
                    PieceType::Knight => KNIGHT_MOBILITY[mobility_count],
                    PieceType::Bishop => BISHOP_MOBILITY[mobility_count],
                    PieceType::Rook => ROOK_MOBILITY[mobility_count],
                    PieceType::Queen => QUEEN_MOBILITY[mobility_count],
                    PieceType::Pawn | PieceType::King => unreachable!(),
                };
        }

        total
    }

    #[inline]
    pub fn passed_pawns<C: ColorParam>(&self, trace: &mut impl Trace) -> S {
        let mut total = S::ZERO;

        for square in self.pawns.passed_pawns[C::INDEX].squares() {
            let square = if C::IS_WHITE { square.flip() } else { square };

            trace.add(|t| t.passed_pawns[square.index()] += C::COEFF);

            total += C::SIGN * PASSED_PAWNS[square.index()];
        }

        total
    }

    // Weirdly, setting #[inline(never)] here compiles the method into a
    // vectorized version which... is faster??
    #[inline(never)]
    pub fn doubled_pawns<C: ColorParam>(&self, trace: &mut impl Trace) -> S {
        let mut total = S::ZERO;
        let pawns = self.pawns.pawns[C::INDEX];

        for file in 0..8 {
            let mask = Bitboard::file_mask(file);
            let pawns_on_file: u32 = (pawns & mask).pop_count();
            let doubled_pawn_count = pawns_on_file.saturating_sub(1);

            trace.add(|t| t.doubled_pawns += C::COEFF * doubled_pawn_count as i16);

            total += C::SIGN * DOUBLED_PAWNS * doubled_pawn_count as i32;
        }

        total
    }

    pub fn isolated_pawns<C: ColorParam>(&self, trace: &mut impl Trace) -> S {
        let isolated = self.pawns.isolated_pawns::<C>();
        let isolated_count = isolated.pop_count();

        trace.add(|t| t.isolated_pawns += C::COEFF * isolated_count as i16);

        C::SIGN * isolated_count as i32 * ISOLATED_PAWNS
    }

    pub fn pawn_shield<C: ColorParam>(&self, trace: &mut impl Trace) -> S {
        let sensible_king_mask = SENSIBLE_KING_MASKS[C::INDEX];
        let king_square = self.kings[C::INDEX];

        if sensible_king_mask.get(king_square) {
            let is_kingside = usize::from(king_square.file() > 3);
            let close_pawns =
                self.pawns.pawns[C::INDEX] & PAWN_SHIELD_CLOSE_MASKS[C::INDEX][is_kingside];
            let far_pawns =
                self.pawns.pawns[C::INDEX] & PAWN_SHIELD_FAR_MASKS[C::INDEX][is_kingside];

            let close_pawns_count = close_pawns.pop_count() as i32;
            let far_pawns_count = far_pawns.pop_count() as i32;

            trace.add(|t| {
                t.pawn_shield_close += C::COEFF * close_pawns_count as i16;
                t.pawn_shield_far += C::COEFF * far_pawns_count as i16;
            });

            C::SIGN * (PAWN_SHIELD_CLOSE * close_pawns_count + PAWN_SHIELD_FAR * far_pawns_count)
        } else {
            S::ZERO
        }
    }

    pub fn pawn_storm<C: ColorParam>(&self, trace: &mut impl Trace) -> S {
        let opponent_idx = C::COLOR.flip().index();
        let opponent_king = self.kings[opponent_idx];

        // Use the passed pawn masks to get the pawns in front of the opponent king.
        let pawn_storm_mask = PASSED_PAWN_MASKS[opponent_idx][opponent_king.index()];
        let storming_pawns = pawn_storm_mask & self.pawns.pawns[C::INDEX];

        let mut total = S::ZERO;

        for pawn in storming_pawns.squares() {
            let distance_idx = match pawn.vertical_distance(opponent_king) {
                x @ 1..=4 => (x - 1) as usize,
                _ => continue,
            };

            trace.add(|t| t.pawn_storm[distance_idx] += C::COEFF);

            total += C::SIGN * PAWN_STORM[distance_idx];
        }

        total
    }
}
