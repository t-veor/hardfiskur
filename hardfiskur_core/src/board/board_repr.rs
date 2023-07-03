use std::ops::{Index, IndexMut, Range};

use super::{bitboard::Bitboard, Color, Move, Piece, PieceType, Square};

#[derive(Debug, Clone, Default)]
pub struct BoardRepr {
    // Repr:
    // 0: All white pieces
    // 1: White pawns
    // 2: White knights
    // 3. White bishops
    // 4. White rooks
    // 5. White queens
    // 6. White kings
    // 7. (unused)
    // 8. All black pieces
    // 9-14: Black piece boards (see above)
    boards: [Bitboard; 15],
}

impl BoardRepr {
    const WHITE_RANGE: Range<usize> = 1..7;
    const BLACK_RANGE: Range<usize> = 9..15;

    pub fn new(board: [Option<Piece>; 64]) -> Self {
        let mut repr = Self::default();

        for (i, piece) in board.into_iter().enumerate() {
            if let Some(piece) = piece {
                let square = Square::from_index_unchecked(i);
                repr[piece].set(square);
                repr[piece.color()].set(square);

                // TODO: Update Zobrist hash
            }
        }

        repr
    }

    pub fn piece_at(&self, square: Square) -> Option<Piece> {
        let mask = Bitboard::from_square(square);

        for i in Self::WHITE_RANGE.chain(Self::BLACK_RANGE) {
            if (mask & self.boards[i]).has_piece() {
                return Piece::try_from_u8(i as u8);
            }
        }

        None
    }

    /// Slightly more optimised version of piece_at if you already know the
    /// color the piece has.
    pub fn piece_with_color_at(&self, color: Color, square: Square) -> Option<Piece> {
        let range = match color {
            Color::White => Self::WHITE_RANGE,
            Color::Black => Self::BLACK_RANGE,
        };
        let mask = Bitboard::from_square(square);

        for i in range {
            if (mask & self.boards[i]).has_piece() {
                return Piece::try_from_u8(i as u8);
            }
        }

        None
    }

    pub fn piece_count(&self, piece_type: PieceType) -> (u32, u32) {
        (
            self[Piece::new(Color::White, piece_type)].pop_count(),
            self[Piece::new(Color::Black, piece_type)].pop_count(),
        )
    }

    pub fn occupied(&self) -> Bitboard {
        self[Color::White] | self[Color::Black]
    }

    pub fn empty(&self) -> Bitboard {
        !self.occupied()
    }

    pub fn boards(&self) -> impl Iterator<Item = (Piece, Bitboard)> + '_ {
        Self::WHITE_RANGE
            .chain(Self::BLACK_RANGE)
            .map(move |i| (Piece::try_from_u8(i as u8).unwrap(), self.boards[i]))
    }

    pub fn pieces(&self) -> impl Iterator<Item = (Piece, Square)> + '_ {
        self.boards().flat_map(|(piece, board)| {
            board
                .bits()
                .map(move |square| (piece, Square::from_u8_unchecked(square)))
        })
    }

    pub fn boards_colored(&self, color: Color) -> impl Iterator<Item = (Piece, Bitboard)> + '_ {
        let range = match color {
            Color::White => Self::WHITE_RANGE,
            Color::Black => Self::BLACK_RANGE,
        };

        range.map(move |i| (Piece::try_from_u8(i as u8).unwrap(), self.boards[i]))
    }

    /// Make the provided move on the board.
    /// No checks are performed to ensure the move is valid.
    /// This method is reversible -- to undo the move, simply call this method
    /// again with the same Move.
    pub fn move_unchecked(&mut self, the_move: Move) {
        let from = the_move.from_square();
        let to = the_move.to_square();
        let piece = the_move.piece();
        let color = piece.color();

        let from_bb = Bitboard::from_square(from);
        let to_bb = Bitboard::from_square(to);
        let from_to_bb = from_bb ^ to_bb;

        self[piece] ^= from_to_bb;
        self[color] ^= from_to_bb;

        // TODO: update Zobrist hash

        if the_move.is_en_passant() {
            let removed_pawn = Square::new_unchecked(from.rank(), to.file());
            let removed_pawn_bb = Bitboard::from_square(removed_pawn);

            self[Piece::new(color.flip(), PieceType::Pawn)] ^= removed_pawn_bb;
            self[color.flip()] ^= removed_pawn_bb;

            // TODO: update Zobrist hash
        } else {
            if let Some(capture) = the_move.captured_piece() {
                self[capture] ^= to_bb;
                self[capture.color()] ^= to_bb;

                // TODO: update Zobrist hash
            }

            if let Some(promote) = the_move.promotion() {
                self[piece] ^= to_bb;
                self[promote] ^= to_bb;

                // TODO: update Zobrist hash
            }

            if the_move.is_castle() {
                let rook_from =
                    Square::new_unchecked(from.rank(), if from.file() < to.file() { 7 } else { 0 });
                let rook_to = Square::new_unchecked(from.rank(), (from.file() + to.file()) / 2);

                let rook_from_bb = Bitboard::from_square(rook_from);
                let rook_to_bb = Bitboard::from_square(rook_to);
                let rook_from_to_bb = rook_from_bb ^ rook_to_bb;

                self[Piece::new(color, PieceType::Rook)] ^= rook_from_to_bb;
                self[color] ^= rook_from_to_bb;

                // TODO: update Zobrist hash
            }
        }
    }
}

impl Index<Piece> for BoardRepr {
    type Output = Bitboard;

    fn index(&self, index: Piece) -> &Self::Output {
        &self.boards[index.get() as usize]
    }
}

impl Index<Color> for BoardRepr {
    type Output = Bitboard;

    fn index(&self, index: Color) -> &Self::Output {
        &self.boards[index as usize]
    }
}

impl IndexMut<Piece> for BoardRepr {
    fn index_mut(&mut self, index: Piece) -> &mut Self::Output {
        &mut self.boards[index.get() as usize]
    }
}

impl IndexMut<Color> for BoardRepr {
    fn index_mut(&mut self, index: Color) -> &mut Self::Output {
        &mut self.boards[index as usize]
    }
}
