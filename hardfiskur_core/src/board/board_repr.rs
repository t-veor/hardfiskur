use std::{
    fmt::{Display, Write},
    ops::{Index, IndexMut, Range},
    str::FromStr,
};

use super::{zobrist::ZobristHash, Bitboard, Color, Move, Piece, PieceType, Square};

/// Represents just the pieces on the board using [`Bitboard`]s.
///
/// Internally, holds a bitboard representing the location of every kind of
/// piece (i.e. every combination of [`Color`] and [`PieceType`]).
///
/// These bitboards can be accessed via indexing by [`Color`] or [`Piece`].
/// For example:
/// ```
/// # use hardfiskur_core::board::{BoardRepr, Color, Piece};
/// fn foo(board_repr: &BoardRepr) {
///     println!("White king: {:?}", board_repr[Piece::WHITE_KING]);
///     println!("Black bishops: {:?}", board_repr[Piece::BLACK_BISHOP]);
///     println!("All white pieces: {:?}", board_repr[Color::White]);
/// }
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq)]
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

    zobrist_hash: ZobristHash,
}

impl BoardRepr {
    const WHITE_RANGE: Range<usize> = 1..7;
    const BLACK_RANGE: Range<usize> = 9..15;

    /// Creates a new [`BoardRepr`] from the provided board state.
    ///
    /// The provided board state should be a slice of [`Option<Piece>`]s,
    /// ordered by increasing file and then rank (i.e. index 0 is a1, index 1 is
    /// b1, index 2 is c1... index 7 is h1, index 8 is a2, index 9 is b2, etc.).
    ///
    /// The provided slice is expected to be of length 64, but if a longer slice
    /// is passed this method will ignore any pieces past index 63, and if a
    /// shorter slice is passed the missing squares are assumed to be empty.
    pub fn new(board: &[Option<Piece>]) -> Self {
        let mut repr = Self::default();

        for (i, &piece) in board.iter().take(64).enumerate() {
            if let Some(piece) = piece {
                let square = Square::from_index_unchecked(i);
                repr[piece].set(square);
                repr[piece.color()].set(square);

                repr.zobrist_hash.toggle_piece(piece, square);
            }
        }

        repr
    }

    /// Returns the piece, if any, on the provided square.
    pub fn piece_at(&self, square: Square) -> Option<Piece> {
        let mask = Bitboard::from_square(square);

        for i in Self::WHITE_RANGE.chain(Self::BLACK_RANGE) {
            if (mask & self.boards[i]).has_piece() {
                return Piece::try_from_u8(i as u8);
            }
        }

        None
    }

    /// Returns the piece with the specified color, if any, on the provided
    /// square.
    ///
    /// This is a slightly more optimised version of [`Self::piece_at`] if you
    /// already know the color the piece has.
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

    /// Returns a tuple (white count, black count) of the provided piece type.
    pub fn piece_count(&self, piece_type: PieceType) -> (u32, u32) {
        (
            self[piece_type.white()].pop_count(),
            self[piece_type.black()].pop_count(),
        )
    }

    /// Returns a bitboard containing all squares that have a piece on them.
    pub fn occupied(&self) -> Bitboard {
        self[Color::White] | self[Color::Black]
    }

    /// Returns a bitboard containning all squares that do not have a piece on
    /// them.
    pub fn empty(&self) -> Bitboard {
        !self.occupied()
    }

    /// Returns an iterator over bitboards for all possible [`Piece`]s.
    pub fn boards(&self) -> impl Iterator<Item = (Piece, Bitboard)> + '_ {
        Self::WHITE_RANGE
            .chain(Self::BLACK_RANGE)
            .map(move |i| (Piece::try_from_u8(i as u8).unwrap(), self.boards[i]))
    }

    /// Returns an iterator for all [`Piece`]s and their corresponding
    /// [`Square`]s.
    ///
    /// Note that the pieces are not necessarily returned in ascending order of
    /// squares.
    pub fn pieces(&self) -> impl Iterator<Item = (Piece, Square)> + '_ {
        self.boards()
            .flat_map(|(piece, board)| board.squares().map(move |square| (piece, square)))
    }

    /// Returns an iterator over bitboards of the given [`Color`].
    pub fn boards_colored(&self, color: Color) -> impl Iterator<Item = (Piece, Bitboard)> + '_ {
        let range = match color {
            Color::White => Self::WHITE_RANGE,
            Color::Black => Self::BLACK_RANGE,
        };

        range.map(move |i| (Piece::try_from_u8(i as u8).unwrap(), self.boards[i]))
    }

    /// Make the provided [`Move`] on the board.
    ///
    /// No checks are performed to ensure the move is valid.
    ///
    /// This method is reversible -- to undo the move, simply call this method
    /// again with the same [`Move`], while the board state is the same as the
    /// state immediately after performing this move (i.e. if you make multiple
    /// moves and want to undo them, you must undo the latest move first and the
    /// earliest move last).
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

        self.zobrist_hash.toggle_piece(piece, from);
        self.zobrist_hash.toggle_piece(piece, to);

        if the_move.is_en_passant() {
            let removed_pawn_square = the_move.en_passant_square();
            let removed_pawn_bb = Bitboard::from_square(removed_pawn_square);

            let opponent_pawn = Piece::pawn(color.flip());

            self[opponent_pawn] ^= removed_pawn_bb;
            self[color.flip()] ^= removed_pawn_bb;

            self.zobrist_hash
                .toggle_piece(opponent_pawn, removed_pawn_square);
        } else {
            if let Some(capture) = the_move.captured_piece() {
                self[capture] ^= to_bb;
                self[capture.color()] ^= to_bb;

                self.zobrist_hash.toggle_piece(capture, to);
            }

            if let Some(promote) = the_move.promotion() {
                self[piece] ^= to_bb;
                self[promote] ^= to_bb;

                self.zobrist_hash.toggle_piece(piece, to);
                self.zobrist_hash.toggle_piece(promote, to);
            }

            if the_move.is_castle() {
                let (rook_from, rook_to) = the_move.castling_rook_squares();
                let rook = Piece::rook(color);

                let rook_from_bb = Bitboard::from_square(rook_from);
                let rook_to_bb = Bitboard::from_square(rook_to);
                let rook_from_to_bb = rook_from_bb ^ rook_to_bb;

                self[rook] ^= rook_from_to_bb;
                self[color] ^= rook_from_to_bb;

                self.zobrist_hash.toggle_piece(rook, rook_from);
                self.zobrist_hash.toggle_piece(rook, rook_to);
            }
        }
    }

    /// Returns the Zobrist hash of the current position. Only includes hash
    /// contributions from pieces on the board.
    ///
    /// Does not include hash contributions from the player to move, castling
    /// rights, or the en passant square.
    pub fn zobrist_hash(&self) -> ZobristHash {
        self.zobrist_hash
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
    /// Intended for internal use. Do not call this directly as you may break
    /// some internal invariants!
    fn index_mut(&mut self, index: Piece) -> &mut Self::Output {
        &mut self.boards[index.get() as usize]
    }
}

impl IndexMut<Color> for BoardRepr {
    /// Intended for internal use. Do not call this directly as you may break
    /// some internal invariants!
    fn index_mut(&mut self, index: Color) -> &mut Self::Output {
        &mut self.boards[index as usize]
    }
}

impl FromStr for BoardRepr {
    type Err = std::convert::Infallible;

    /// Parses a string into a [`BoardRepr`]. This is intended to make
    /// specifying boards easier for tests.
    ///
    /// Boards should look something like the following:
    /// ```txt
    /// r n b q k b n r
    /// p p p p p p p p
    /// . . . . . . . .
    /// . . . . . . . .
    /// . . . . P . . .
    /// . . . . . . . .
    /// P P P P . P P P
    /// R N B Q K B N R
    /// ```
    ///
    /// Characters are interpreted as squares starting from the top left (a8),
    /// by file and then by rank.
    ///
    /// The parsing is very permissive:
    /// * `.` and `\u{00a0}` (non-breaking space) are interpreted as an empty
    ///   square.
    /// * FEN characters (`PNBRQKpnbrqk`) are interpreted as the corresponding
    ///   piece.
    /// * All other characters are ignored.
    ///
    /// If less than 64 empty squares/pieces are encountered, the remaining
    /// squares are assumed empty. If more are encountered, the extra squares
    /// are ignored.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut board = vec![None; 64];

        let mut square_iter = (0..8)
            .rev()
            .flat_map(|rank| (0..8).map(move |file| Square::new(rank, file).unwrap()))
            .peekable();

        for c in s.chars() {
            if c == '.' || c == '\u{00a0}' {
                square_iter.next();
            } else if let (Some(square), Some(piece)) =
                (square_iter.peek(), Piece::try_from_fen_char(c))
            {
                board[square.index()] = Some(piece);
                square_iter.next();
            }
        }

        Ok(Self::new(&board))
    }
}

impl Display for BoardRepr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for rank in (0..8).rev() {
            f.write_str("+---+---+---+---+---+---+---+---+\n")?;

            for file in 0..8 {
                let piece = self.piece_at(Square::new_unchecked(rank, file));
                let fill_char = if (rank + file) % 2 == 0 {
                    '.'
                } else {
                    '\u{00a0}'
                };

                f.write_char('|')?;
                if let Some(piece) = piece {
                    if piece.is_black() {
                        f.write_char('*')?;
                    } else {
                        f.write_char(' ')?;
                    }

                    f.write_fmt(format_args!("{} ", piece.as_fen_char()))?;
                } else {
                    f.write_fmt(format_args!(" {fill_char} "))?;
                }
            }

            f.write_fmt(format_args!("| {}\n", rank + 1))?;
        }

        f.write_str("+---+---+---+---+---+---+---+---+\n")?;
        f.write_str("  a   b   c   d   e   f   g   h")?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;
    use std::str::FromStr;

    use super::*;

    impl BoardRepr {
        fn consistency_check(&self) {
            let check = || {
                let mut white_pieces = Bitboard::EMPTY;

                for (_, board) in self.boards_colored(Color::White) {
                    if (white_pieces & board).has_piece() {
                        return false;
                    }

                    white_pieces |= board;
                }

                if white_pieces != self[Color::White] {
                    return false;
                }

                let mut black_pieces = Bitboard::EMPTY;

                for (_, board) in self.boards_colored(Color::Black) {
                    if (black_pieces & board).has_piece() {
                        return false;
                    }

                    black_pieces |= board;
                }

                if black_pieces != self[Color::Black] {
                    return false;
                }

                if !(white_pieces & black_pieces).is_empty() {
                    return false;
                }

                let mut zobrist_hash = ZobristHash::default();
                for (piece, square) in self.pieces() {
                    zobrist_hash.toggle_piece(piece, square);
                }

                if zobrist_hash != self.zobrist_hash {
                    return false;
                }

                true
            };

            assert!(check(), "BoardRepr became inconsistent, {self:?}");
        }
    }

    fn b(sq: &str) -> Bitboard {
        Bitboard::from_square(sq.parse().unwrap())
    }

    fn starting_position() -> BoardRepr {
        "
            rnbqkbnr
            pppppppp
            ........
            ........
            ........
            ........
            PPPPPPPP
            RNBQKBNR
        "
        .parse()
        .unwrap()
    }

    #[test]
    fn board_repr_new_list_too_short() {
        let board = BoardRepr::new(&[
            Some(Piece::WHITE_ROOK),
            Some(Piece::BLACK_KING),
            None,
            Some(Piece::BLACK_BISHOP),
        ]);

        let mut pieces = board.pieces().collect::<Vec<_>>();
        pieces.sort_by_key(|&(_piece, square)| square);

        assert_eq!(
            pieces,
            vec![
                (Piece::WHITE_ROOK, "a1".parse().unwrap()),
                (Piece::BLACK_KING, "b1".parse().unwrap()),
                (Piece::BLACK_BISHOP, "d1".parse().unwrap()),
            ]
        )
    }

    #[test]
    fn board_repr_new_list_too_long() {
        let board = BoardRepr::new(
            &vec![
                vec![Some(Piece::BLACK_KNIGHT), Some(Piece::WHITE_QUEEN)],
                vec![None; 62],
                vec![Some(Piece::WHITE_KING)],
            ]
            .concat(),
        );

        let mut pieces = board.pieces().collect::<Vec<_>>();
        pieces.sort_by_key(|&(_piece, square)| square);

        assert_eq!(
            pieces,
            vec![
                (Piece::BLACK_KNIGHT, "a1".parse().unwrap()),
                (Piece::WHITE_QUEEN, "b1".parse().unwrap()),
            ]
        )
    }

    #[test]
    fn board_repr_piece_at() {
        let board = starting_position();

        assert_eq!(
            board.piece_at("d1".parse().unwrap()),
            Some(Piece::WHITE_QUEEN)
        );
        assert_eq!(
            board.piece_at("h1".parse().unwrap()),
            Some(Piece::WHITE_ROOK)
        );
        assert_eq!(
            board.piece_at("b2".parse().unwrap()),
            Some(Piece::WHITE_PAWN)
        );

        assert_eq!(
            board.piece_at("b8".parse().unwrap()),
            Some(Piece::BLACK_KNIGHT)
        );
        assert_eq!(
            board.piece_at("c8".parse().unwrap()),
            Some(Piece::BLACK_BISHOP)
        );
        assert_eq!(
            board.piece_at("h7".parse().unwrap()),
            Some(Piece::BLACK_PAWN)
        );

        assert_eq!(board.piece_at("e4".parse().unwrap()), None);
        assert_eq!(board.piece_at("a5".parse().unwrap()), None);
        assert_eq!(board.piece_at("c6".parse().unwrap()), None);
        assert_eq!(board.piece_at("f3".parse().unwrap()), None);
    }

    #[test]
    fn board_repr_piece_with_color_at() {
        let board = starting_position();

        assert_eq!(
            board.piece_with_color_at(Color::White, "d1".parse().unwrap()),
            Some(Piece::WHITE_QUEEN)
        );
        assert_eq!(
            board.piece_with_color_at(Color::Black, "d1".parse().unwrap()),
            None,
        );

        assert_eq!(
            board.piece_with_color_at(Color::White, "h7".parse().unwrap()),
            None
        );
        assert_eq!(
            board.piece_with_color_at(Color::Black, "h7".parse().unwrap()),
            Some(Piece::BLACK_PAWN)
        );

        assert_eq!(
            board.piece_with_color_at(Color::White, "e4".parse().unwrap()),
            None
        );
        assert_eq!(
            board.piece_with_color_at(Color::Black, "e4".parse().unwrap()),
            None
        );
    }

    #[test]
    pub fn board_repr_piece_count() {
        let board = BoardRepr::from_str(
            "
                .....B..
                ......P.
                .p......
                ........
                .N....q.
                kP......
                ..K.....
                ........",
        )
        .unwrap();

        assert_eq!(board.piece_count(PieceType::Pawn), (2, 1));
        assert_eq!(board.piece_count(PieceType::Knight), (1, 0));
        assert_eq!(board.piece_count(PieceType::Bishop), (1, 0));
        assert_eq!(board.piece_count(PieceType::Rook), (0, 0));
        assert_eq!(board.piece_count(PieceType::Queen), (0, 1));
        assert_eq!(board.piece_count(PieceType::King), (1, 1));
    }

    #[test]
    pub fn board_repr_occupied_empty() {
        let test_str = "
            .....B..
            ......P.
            .p......
            ........
            .N....q.
            kP......
            ..K.....
            ........";

        let expected_occupied = Bitboard::from_str(test_str).unwrap();
        let expected_empty = !expected_occupied;

        let board = BoardRepr::from_str(test_str).unwrap();

        assert_eq!(board.occupied(), expected_occupied);
        assert_eq!(board.empty(), expected_empty);
    }

    #[test]
    pub fn board_repr_boards() {
        let test_str = "
            r...k..r
            ..qn.pb.
            .p..p.np
            ...pPb..
            .....B..
            .N.N....
            PPP.B.PP
            R..Q.RK.";

        let board = BoardRepr::from_str(test_str).unwrap();
        let boards = board.boards().collect::<Vec<_>>();

        assert_eq!(
            boards,
            vec![
                (
                    Piece::WHITE_PAWN,
                    b("a2") | b("b2") | b("c2") | b("e5") | b("g2") | b("h2")
                ),
                (Piece::WHITE_KNIGHT, b("b3") | b("d3")),
                (Piece::WHITE_BISHOP, b("e2") | b("f4")),
                (Piece::WHITE_ROOK, b("a1") | b("f1")),
                (Piece::WHITE_QUEEN, b("d1")),
                (Piece::WHITE_KING, b("g1")),
                (
                    Piece::BLACK_PAWN,
                    b("b6") | b("d5") | b("e6") | b("f7") | b("h6")
                ),
                (Piece::BLACK_KNIGHT, b("d7") | b("g6")),
                (Piece::BLACK_BISHOP, b("f5") | b("g7")),
                (Piece::BLACK_ROOK, b("a8") | b("h8")),
                (Piece::BLACK_QUEEN, b("c7")),
                (Piece::BLACK_KING, b("e8")),
            ]
        );
    }

    #[test]
    fn board_repr_pieces() {
        let test_str = "
            .....B..
            ......P.
            .p......
            ........
            .N....q.
            kP......
            ..K.....
            ........";

        let board = BoardRepr::from_str(test_str).unwrap();

        let mut pieces = board.pieces().collect::<Vec<_>>();
        pieces.sort_by_key(|&(_piece, square)| square);

        assert_eq!(
            pieces,
            vec![
                (Piece::WHITE_KING, "c2".parse().unwrap()),
                (Piece::BLACK_KING, "a3".parse().unwrap()),
                (Piece::WHITE_PAWN, "b3".parse().unwrap()),
                (Piece::WHITE_KNIGHT, "b4".parse().unwrap()),
                (Piece::BLACK_QUEEN, "g4".parse().unwrap()),
                (Piece::BLACK_PAWN, "b6".parse().unwrap()),
                (Piece::WHITE_PAWN, "g7".parse().unwrap()),
                (Piece::WHITE_BISHOP, "f8".parse().unwrap()),
            ]
        );
    }

    #[test]
    fn board_repr_boards_colored() {
        let test_str = "
            r...k..r
            ..qn.pb.
            .p..p.np
            ...pPb..
            .....B..
            .N.N....
            PPP.B.PP
            R..Q.RK.";

        let board = BoardRepr::from_str(test_str).unwrap();

        assert_eq!(
            board.boards_colored(Color::White).collect::<Vec<_>>(),
            vec![
                (
                    Piece::WHITE_PAWN,
                    b("a2") | b("b2") | b("c2") | b("e5") | b("g2") | b("h2")
                ),
                (Piece::WHITE_KNIGHT, b("b3") | b("d3")),
                (Piece::WHITE_BISHOP, b("e2") | b("f4")),
                (Piece::WHITE_ROOK, b("a1") | b("f1")),
                (Piece::WHITE_QUEEN, b("d1")),
                (Piece::WHITE_KING, b("g1")),
            ]
        );

        assert_eq!(
            board.boards_colored(Color::Black).collect::<Vec<_>>(),
            vec![
                (
                    Piece::BLACK_PAWN,
                    b("b6") | b("d5") | b("e6") | b("f7") | b("h6")
                ),
                (Piece::BLACK_KNIGHT, b("d7") | b("g6")),
                (Piece::BLACK_BISHOP, b("f5") | b("g7")),
                (Piece::BLACK_ROOK, b("a8") | b("h8")),
                (Piece::BLACK_QUEEN, b("c7")),
                (Piece::BLACK_KING, b("e8")),
            ]
        );
    }

    #[test]
    pub fn board_repr_index() {
        let test_str = "
            r...k..r
            ..qn.pb.
            .p..p.np
            ...pPb..
            .....B..
            .N.N....
            PPP.B.PP
            R..Q.RK.";

        let board = BoardRepr::from_str(test_str).unwrap();

        assert_eq!(
            board[Piece::WHITE_PAWN],
            b("a2") | b("b2") | b("c2") | b("e5") | b("g2") | b("h2")
        );
        assert_eq!(board[Piece::WHITE_KNIGHT], b("b3") | b("d3"));
        assert_eq!(board[Piece::WHITE_BISHOP], b("e2") | b("f4"));
        assert_eq!(board[Piece::WHITE_ROOK], b("a1") | b("f1"));
        assert_eq!(board[Piece::WHITE_QUEEN], b("d1"));
        assert_eq!(board[Piece::WHITE_KING], b("g1"));

        assert_eq!(
            board[Color::White],
            Bitboard::from_str(
                "
                    ........
                    ........
                    ........
                    ....P...
                    .....B..
                    .N.N....
                    PPP.B.PP
                    R..Q.RK.",
            )
            .unwrap()
        );

        assert_eq!(
            board[Piece::BLACK_PAWN],
            b("b6") | b("d5") | b("e6") | b("f7") | b("h6")
        );
        assert_eq!(board[Piece::BLACK_KNIGHT], b("d7") | b("g6"));
        assert_eq!(board[Piece::BLACK_BISHOP], b("f5") | b("g7"));
        assert_eq!(board[Piece::BLACK_ROOK], b("a8") | b("h8"));
        assert_eq!(board[Piece::BLACK_QUEEN], b("c7"));
        assert_eq!(board[Piece::BLACK_KING], b("e8"));

        assert_eq!(
            board[Color::Black],
            Bitboard::from_str(
                "
                    r...k..r
                    ..qn.pb.
                    .p..p.np
                    ...p.b..
                    ........
                    ........
                    ........
                    ........",
            )
            .unwrap()
        );
    }

    const MODIFIED_KIWIPETE: &str = "
        r...k..r
        p..pqpP.
        bn..pnp.
        ..pPN...
        Pp..P...
        ..N..Q.p
        .PPBBPP.
        R...K..R
    ";

    const QUIET_MOVES: &[Move] = &[
        Move::builder(Square::A6, Square::D3, Piece::BLACK_BISHOP).build(),
        Move::builder(Square::F3, Square::G3, Piece::WHITE_QUEEN).build(),
        Move::builder(Square::G2, Square::G4, Piece::WHITE_PAWN)
            .is_double_pawn_push()
            .build(),
    ];

    #[test]
    fn board_move_unchecked_quiet_moves() {
        let board = BoardRepr::from_str(MODIFIED_KIWIPETE).unwrap();

        for &the_move in QUIET_MOVES {
            let mut moved_board = board.clone();
            moved_board.move_unchecked(the_move);

            assert_eq!(
                moved_board[the_move.piece()],
                board[the_move.piece()]
                    .without(Bitboard::from_square(the_move.from_square()))
                    .or(Bitboard::from_square(the_move.to_square()))
            );
            assert_eq!(
                moved_board[the_move.piece().color()],
                board[the_move.piece().color()]
                    .without(Bitboard::from_square(the_move.from_square()))
                    .or(Bitboard::from_square(the_move.to_square()))
            );

            moved_board.consistency_check();

            moved_board.move_unchecked(the_move);

            assert_eq!(moved_board, board);
        }
    }

    const REGULAR_CAPTURES: &[Move] = &[
        Move::builder(Square::B4, Square::C3, Piece::BLACK_PAWN)
            .captures(Piece::WHITE_KNIGHT)
            .build(),
        Move::builder(Square::E5, Square::D7, Piece::WHITE_KNIGHT)
            .captures(Piece::BLACK_PAWN)
            .build(),
    ];

    #[test]
    fn board_move_unchecked_regular_captures() {
        let board = BoardRepr::from_str(MODIFIED_KIWIPETE).unwrap();

        for &the_move in REGULAR_CAPTURES {
            let mut moved_board = board.clone();
            moved_board.move_unchecked(the_move);

            assert_eq!(
                moved_board[the_move.piece()],
                board[the_move.piece()]
                    .without(Bitboard::from_square(the_move.from_square()))
                    .or(Bitboard::from_square(the_move.to_square()))
            );
            assert_eq!(
                moved_board[the_move.piece().color()],
                board[the_move.piece().color()]
                    .without(Bitboard::from_square(the_move.from_square()))
                    .or(Bitboard::from_square(the_move.to_square()))
            );

            assert_eq!(
                moved_board[the_move.captured_piece().unwrap()],
                board[the_move.captured_piece().unwrap()]
                    .without(Bitboard::from_square(the_move.to_square()))
            );
            assert_eq!(
                moved_board[the_move.captured_piece().unwrap().color()],
                board[the_move.captured_piece().unwrap().color()]
                    .without(Bitboard::from_square(the_move.to_square()))
            );

            moved_board.consistency_check();

            moved_board.move_unchecked(the_move);

            assert_eq!(moved_board, board);
        }
    }

    const PROMOTIONS: &[Move] = &[
        Move::builder(Square::G7, Square::G8, Piece::WHITE_PAWN)
            .promotes_to(PieceType::Knight)
            .build(),
        Move::builder(Square::G7, Square::H8, Piece::WHITE_PAWN)
            .captures(Piece::BLACK_ROOK)
            .promotes_to(PieceType::Queen)
            .build(),
    ];

    #[test]
    fn board_move_unchecked_promotions() {
        let board = BoardRepr::from_str(MODIFIED_KIWIPETE).unwrap();

        for &the_move in PROMOTIONS {
            let mut moved_board = board.clone();
            moved_board.move_unchecked(the_move);

            assert_eq!(
                moved_board[the_move.piece()],
                board[the_move.piece()].without(Bitboard::from_square(the_move.from_square()))
            );
            assert_eq!(
                moved_board[the_move.promotion().unwrap()],
                board[the_move.promotion().unwrap()]
                    .or(Bitboard::from_square(the_move.to_square()))
            );
            assert_eq!(
                moved_board[the_move.piece().color()],
                board[the_move.piece().color()]
                    .without(Bitboard::from_square(the_move.from_square()))
                    .or(Bitboard::from_square(the_move.to_square()))
            );

            if let Some(captured_piece) = the_move.captured_piece() {
                assert_eq!(
                    moved_board[captured_piece],
                    board[captured_piece].without(Bitboard::from_square(the_move.to_square()))
                );
                assert_eq!(
                    moved_board[captured_piece.color()],
                    board[captured_piece.color()]
                        .without(Bitboard::from_square(the_move.to_square()))
                );
            }

            moved_board.consistency_check();

            moved_board.move_unchecked(the_move);

            assert_eq!(moved_board, board);
        }
    }

    const CASTLES: &[(Move, Square, Square)] = &[
        (
            Move::builder(Square::E1, Square::G1, Piece::WHITE_KING)
                .is_castle()
                .build(),
            Square::WHITE_KINGSIDE_ROOK,
            Square::F1,
        ),
        (
            Move::builder(Square::E1, Square::C1, Piece::WHITE_KING)
                .is_castle()
                .build(),
            Square::WHITE_QUEENSIDE_ROOK,
            Square::D1,
        ),
        (
            Move::builder(Square::E8, Square::G8, Piece::BLACK_KING)
                .is_castle()
                .build(),
            Square::BLACK_KINGSIDE_ROOK,
            Square::F8,
        ),
        (
            Move::builder(Square::E8, Square::C8, Piece::BLACK_KING)
                .is_castle()
                .build(),
            Square::BLACK_QUEENSIDE_ROOK,
            Square::D8,
        ),
    ];

    #[test]
    fn board_move_unchecked_castles() {
        let board = BoardRepr::from_str(MODIFIED_KIWIPETE).unwrap();

        for &(the_move, rook_from, rook_to) in CASTLES {
            let mut moved_board = board.clone();
            moved_board.move_unchecked(the_move);

            assert_eq!(
                moved_board[the_move.piece()],
                board[the_move.piece()]
                    .without(Bitboard::from_square(the_move.from_square()))
                    .or(Bitboard::from_square(the_move.to_square()))
            );
            let rook = PieceType::Rook.with_color(the_move.piece().color());
            assert_eq!(
                moved_board[rook],
                board[rook]
                    .without(Bitboard::from_square(rook_from))
                    .or(Bitboard::from_square(rook_to))
            );

            assert_eq!(
                moved_board[the_move.piece().color()],
                board[the_move.piece().color()]
                    .without(
                        Bitboard::from_square(the_move.from_square())
                            | Bitboard::from_square(rook_from)
                    )
                    .or(Bitboard::from_square(the_move.to_square())
                        | Bitboard::from_square(rook_to))
            );

            moved_board.consistency_check();

            moved_board.move_unchecked(the_move);

            assert_eq!(moved_board, board);
        }
    }

    const EN_PASSANT_CAPTURES: &[(Move, Square)] = &[
        (
            Move::builder(Square::D5, Square::C6, Piece::WHITE_PAWN)
                .captures(Piece::BLACK_PAWN)
                .is_en_passant()
                .build(),
            Square::C5,
        ),
        (
            Move::builder(Square::B4, Square::A3, Piece::BLACK_PAWN)
                .captures(Piece::WHITE_PAWN)
                .is_en_passant()
                .build(),
            Square::A4,
        ),
    ];

    #[test]
    fn board_move_unchecked_en_passant() {
        let board = BoardRepr::from_str(MODIFIED_KIWIPETE).unwrap();

        for &(the_move, captured_pawn_square) in EN_PASSANT_CAPTURES {
            let mut moved_board = board.clone();
            moved_board.move_unchecked(the_move);

            assert_eq!(
                moved_board[the_move.piece()],
                board[the_move.piece()]
                    .without(Bitboard::from_square(the_move.from_square()))
                    .or(Bitboard::from_square(the_move.to_square()))
            );
            assert_eq!(
                moved_board[the_move.piece().color()],
                board[the_move.piece().color()]
                    .without(Bitboard::from_square(the_move.from_square()))
                    .or(Bitboard::from_square(the_move.to_square()))
            );

            assert_eq!(
                moved_board[the_move.captured_piece().unwrap()],
                board[the_move.captured_piece().unwrap()]
                    .without(Bitboard::from_square(captured_pawn_square))
            );
            assert_eq!(
                moved_board[the_move.captured_piece().unwrap().color()],
                board[the_move.captured_piece().unwrap().color()]
                    .without(Bitboard::from_square(captured_pawn_square))
            );

            moved_board.consistency_check();

            moved_board.move_unchecked(the_move);

            assert_eq!(moved_board, board);
        }
    }

    #[test]
    fn board_repr_display_from_str() {
        let board = starting_position();
        let parsed = format!("{board}").parse().unwrap();

        assert_eq!(board, parsed);
    }
}
