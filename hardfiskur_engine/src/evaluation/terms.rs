use hardfiskur_core::board::{Bitboard, Color, Piece, PieceType, Square};

use super::{
    lookups::{PAWN_SHIELD_CLOSE_MASKS, PAWN_SHIELD_FAR_MASKS, SENSIBLE_KING_MASKS},
    packed_score::S,
    parameters::*,
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
    pub fn open_file_bonus<C: ColorParam>(
        &self,
        piece_type: PieceType,
        square: Square,
        trace: &mut impl Trace,
    ) -> S {
        let idx = match piece_type {
            PieceType::Rook => 0,
            PieceType::Queen => 1,
            PieceType::King => 2,
            _ => return S::ZERO,
        };

        let semi_open = self.pawns.semi_open_files[C::INDEX].get(square);
        let fully_open =
            semi_open && self.pawns.semi_open_files[C::COLOR.flip().index()].get(square);

        trace.add(|t| {
            if fully_open {
                t.open_file_bonuses[idx] += C::COEFF;
            } else if semi_open {
                t.semi_open_file_bonuses[idx] += C::COEFF;
            }
        });

        if fully_open {
            C::SIGN * OPEN_FILE_BONUSES[idx]
        } else if semi_open {
            C::SIGN * SEMI_OPEN_FILE_BONUSES[idx]
        } else {
            S::ZERO
        }
    }

    #[inline]
    pub fn mobility_and_king_zone_attacks<C: ColorParam, P: PieceTypeParam>(
        &self,
        trace: &mut impl Trace,
    ) -> S {
        const {
            assert!(
                !matches!(P::PIECE_TYPE, PieceType::Pawn | PieceType::King),
                "Can't call mobility_and_king_zone_attacks() with Pawn or King!"
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
            let attack_bb = match P::PIECE_TYPE {
                PieceType::Knight => self.lookups.get_knight_moves(square),
                PieceType::Bishop => self.lookups.get_bishop_attacks(self.occupied, square),
                PieceType::Rook => self.lookups.get_rook_attacks(self.occupied, square),
                PieceType::Queen => self.lookups.get_queen_attacks(self.occupied, square),
                PieceType::Pawn | PieceType::King => unreachable!(),
            };

            // Mobility
            let mobility_bb = attack_bb & mobility_squares;
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

            // King zone attacks
            let king_zone_attacks = attack_bb & self.king_zones[C::Flip::INDEX];
            let king_zone_attack_count = king_zone_attacks.pop_count() as i32;

            trace
                .add(|t| t.king_zone_attacks[P::INDEX] += C::COEFF * king_zone_attack_count as i16);

            total += C::SIGN * KING_ZONE_ATTACKS[P::INDEX] * king_zone_attack_count;
        }

        total
    }

    pub fn virtual_mobility<C: ColorParam>(&self, trace: &mut impl Trace) -> S {
        // Pretend the king is a queen and apply a malus based on how many
        // squares the virtual queen can see, as an estimate of how vulnerable
        // the king is to sliding piece attacks.
        let king_square = self.kings[C::INDEX];
        let occupied = self.occupied;
        let our_pieces = self.board.get_bitboard_for_color(C::COLOR);

        let virtual_queen_attacks = self
            .lookups
            .get_queen_attacks(occupied, king_square)
            .without(our_pieces);
        let virtual_mobility = virtual_queen_attacks.pop_count() as usize;

        trace.add(|t| t.virtual_mobility[virtual_mobility] += C::COEFF);

        C::SIGN * VIRTUAL_MOBILITY[virtual_mobility]
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

    pub fn phalanx_pawns<C: ColorParam>(&self, trace: &mut impl Trace) -> S {
        let pawns = self.pawns.pawns[C::INDEX];
        let phalanx_pawns = pawns & (pawns.step_east() | pawns.step_west());
        let phalanx_count = phalanx_pawns.pop_count();

        trace.add(|t| t.phalanx_pawns += C::COEFF * phalanx_count as i16);

        C::SIGN * phalanx_count as i32 * PHALANX_PAWNS
    }

    pub fn protected_pawns<C: ColorParam>(&self, trace: &mut impl Trace) -> S {
        let protected_pawns = self.pawns.pawns[C::INDEX] & self.pawns.pawn_attacks[C::INDEX];
        let protected_count = protected_pawns.pop_count();

        trace.add(|t| t.protected_pawns += C::COEFF * protected_count as i16);

        C::SIGN * protected_count as i32 * PROTECTED_PAWNS
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

    pub fn knight_outposts<C: ColorParam>(&self, trace: &mut impl Trace) -> S {
        let knights_in_outposts = self.pawns.outposts[C::INDEX]
            & self.board.get_bitboard_for_piece(Piece::knight(C::COLOR));
        let count = knights_in_outposts.pop_count() as i32;

        trace.add(|t| t.knight_outposts += C::COEFF * count as i16);

        C::SIGN * KNIGHT_OUTPOSTS * count
    }

    pub fn bishop_outposts<C: ColorParam>(&self, trace: &mut impl Trace) -> S {
        let bishops_in_outposts = self.pawns.outposts[C::INDEX]
            & self.board.get_bitboard_for_piece(Piece::bishop(C::COLOR));
        let count = bishops_in_outposts.pop_count() as i32;

        trace.add(|t| t.bishop_outposts += C::COEFF * count as i16);

        C::SIGN * BISHOP_OUTPOSTS * count
    }
}
