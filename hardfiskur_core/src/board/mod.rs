//! Structs and functions related to to the board representation.

mod bitboard;
mod board_repr;
mod castling;
mod fen;
mod move_repr;
mod piece;
mod san;
mod square;
mod uci_move;
mod zobrist;

use std::fmt::Display;

pub use bitboard::Bitboard;
pub use board_repr::BoardRepr;
pub use castling::Castling;
pub use fen::FenParseError;
pub use move_repr::{Move, MoveBuilder, MoveFlags, OptionalMove};
pub use piece::{Color, Piece, PieceType};
pub use san::SAN;
pub use square::{ParseSquareError, Square};
pub use uci_move::{ParseUCIMoveError, UCIMove};
pub use zobrist::ZobristHash;

use crate::move_gen::{self, MoveGenFlags, MoveGenResult, MoveGenerator, MoveVec};

pub const STARTING_POSITION_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

/// State of play for the board.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BoardState {
    /// The player to move has legal moves available, and the game is not drawn.
    InPlay { checkers: u32 },
    /// The game is drawn.
    Draw(DrawReason),
    /// The game is over with a win for the specified player.
    Win(Color),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DrawReason {
    Stalemate,
    ThreeFoldRepetition,
    FiftyMoveRule,
    InsufficientMaterial,
}

/// Holds relevant information needed to undo a move.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct UnmakeData {
    the_move: Option<Move>,
    castling: Castling,
    en_passant: Option<Square>,
    halfmove_clock: u32,
    zobrist_hash: ZobristHash,
}

/// Represents the current game state.
///
/// Contains a bitboard representation of the board, along with other
/// information such as move history, castling rights, etc.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Board {
    board: BoardRepr,
    to_move: Color,
    castling: Castling,
    en_passant: Option<Square>,
    halfmove_clock: u32,
    fullmoves: u32,

    move_history: Vec<UnmakeData>,
    zobrist_hash: ZobristHash,
}

impl Board {
    /// Create a new [`Board`].
    ///
    /// # Arguments
    ///
    /// * `board` - a slice of [`Option<Piece>`]s, ordered by increasing file
    ///   and then rank. See [`BoardRepr::new`] for more details.
    /// * `to_move` - the [`Color`] of the current player.
    /// * `castling` - castling rights for both players, see [`Castling`]
    /// * `en_passant` - set if a double pawn push was made on the immediate
    ///    previous ply and the current player has the option to capture en
    ///    passant.
    ///
    ///    In this case, `en_passant` should be `Some(square)`, where `square`
    ///    is the square behind the double-moved pawn that the current player's
    ///    pawn will land on if they choose to en passant.
    /// * `halfmove_clock` - Half-move clock, represents the number of plies
    ///    since the last capture or pawn-push. This is used for tracking the
    ///    50-move draw rule.
    /// * `fullmoves` - Number of full moves (2 plies, a move by white and a
    ///    move by black) since the start of the game. Starts at 1.
    pub fn new(
        board: &[Option<Piece>],
        to_move: Color,
        castling: Castling,
        en_passant: Option<Square>,
        halfmove_clock: u32,
        fullmoves: u32,
    ) -> Self {
        let board = BoardRepr::new(board);

        let zobrist_hash =
            board.zobrist_hash() ^ Self::non_board_hash(to_move, castling, en_passant);

        Self {
            board,
            to_move,
            castling,
            en_passant,
            halfmove_clock,
            fullmoves,

            move_history: Vec::new(),
            zobrist_hash,
        }
    }

    /// Returns a [`Board`] representing the starting position of a standard
    /// chess game.
    pub fn starting_position() -> Self {
        Self::try_parse_fen(STARTING_POSITION_FEN).unwrap()
    }

    /// Returns the [`Color`] of the current player.
    pub fn to_move(&self) -> Color {
        self.to_move
    }

    /// Returns the castling rights in the current position.
    ///
    /// See [`Castling`] documentation for more details.
    pub fn castling(&self) -> Castling {
        self.castling
    }

    /// Returns the square on which a pawn may be captured en passant if the
    /// previous move was a double pawn push.
    ///
    /// For example, if the previous move was a white pawn move from E2 to E4,
    /// then this method will return `Some(Square::E3)`. If the previous move
    /// was not a double pawn push, then this method will return `None`.
    pub fn en_passant(&self) -> Option<Square> {
        self.en_passant
    }

    /// Returns the current half-move clock of the current position.
    ///
    /// The half-move clock is used to implement the fifty-move rule, where
    /// after fifty consecutive "full" moves without a capture or a pawn move,
    /// the player to move can immediately claim a draw (assuming they are not
    /// in checkmate).
    ///
    /// This method returns the number of "half" moves or plies since the last
    /// capture or pawn move. When a piece is captured or a pawn is moved, this
    /// counter is reset to 0. Any other moves increment this counter. Since
    /// there are two plies in a "full" move, the game is considered drawn when
    /// this counter reaches 100.
    pub fn halfmove_clock(&self) -> u32 {
        self.halfmove_clock
    }

    /// Returns the "full" move count from the start of the game.
    ///
    /// A full move is two plies, i.e. a white move and a black move.
    pub fn fullmoves(&self) -> u32 {
        self.fullmoves
    }

    pub fn zobrist_hash(&self) -> ZobristHash {
        self.zobrist_hash
    }

    /// Returns an iterator over all the pieces on the board and the square
    /// they're on.
    ///
    /// The pieces are not guaranteed to be returned in any particular order.
    pub fn pieces(&self) -> impl Iterator<Item = (Piece, Square)> + '_ {
        self.board.pieces()
    }

    /// Returns the piece that's on a specific square.
    pub fn get_piece(&self, square: Square) -> Option<Piece> {
        self.board.piece_at(square)
    }

    /// Generate all the possible legal moves in the current position.
    pub fn legal_moves(&self) -> MoveVec {
        let mut moves = MoveVec::new();
        self.legal_moves_ex(MoveGenFlags::default(), &mut moves);
        moves
    }

    /// Generate all the possible legal moves in the current position, and
    /// some additional information about the position:
    /// * number of enemy pieces checking the king (of the side to move)
    /// * whether en passant capture is available or not
    pub fn legal_moves_and_meta(&self) -> (MoveVec, MoveGenResult) {
        let mut moves = MoveVec::new();
        let result = self.legal_moves_ex(MoveGenFlags::default(), &mut moves);
        (moves, result)
    }

    /// More customisable version of [`Self::legal_moves_and_meta`], allowing
    /// you to pass in [`MoveGenFlags`] to control whether captures or pushes
    /// should be generated, and specify where moves should be output into via
    /// `out_moves`.
    pub fn legal_moves_ex(&self, flags: MoveGenFlags, out_moves: &mut MoveVec) -> MoveGenResult {
        MoveGenerator::new(
            &self.board,
            self.to_move,
            self.en_passant,
            self.castling,
            flags,
            out_moves,
        )
        .legal_moves()
    }

    /// Returns the current state of the game, i.e. whether it is still in play,
    /// a win for either player or drawn. See [`BoardState`] for the possible
    /// states of a game.
    ///
    /// This method is rather slow and is intended to conveniently calculate the
    /// state of a game for display purposes. Within an engine you would most
    /// likely attempt to calculate the state of the game using already computed
    /// information (e.g. legal moves from the current position) to avoid extra
    /// recomputation.
    pub fn state(&self) -> BoardState {
        let (legal_moves, move_gen_result) = self.legal_moves_and_meta();
        let in_check = move_gen_result.checker_count > 0;

        if !legal_moves.is_empty() {
            if self.halfmove_clock >= 100 {
                BoardState::Draw(DrawReason::FiftyMoveRule)
            } else if self.check_draw_by_insufficient_material() {
                BoardState::Draw(DrawReason::InsufficientMaterial)
            } else if self.check_draw_by_repetition() {
                BoardState::Draw(DrawReason::ThreeFoldRepetition)
            } else {
                BoardState::InPlay {
                    checkers: move_gen_result.checker_count,
                }
            }
        } else if in_check {
            BoardState::Win(self.to_move.flip())
        } else {
            BoardState::Draw(DrawReason::Stalemate)
        }
    }

    /// Attempts to find a legal move matching the provided parameters.
    ///
    /// `promotion` should be `None` unless the move involves a pawn moving to
    /// the back rank, in which case it should be `Some` of a valid promotion
    /// piece type.
    pub fn get_move(&self, from: Square, to: Square, promotion: Option<PieceType>) -> Option<Move> {
        let legal_moves = self.legal_moves();

        legal_moves.into_iter().find(|m| {
            if m.from_square() == from
                && m.to_square() == to
                && m.promotion().map(|piece| piece.piece_type()) == promotion
            {
                return true;
            }

            // Special case -- a move where the king "captures" its own rook can
            // be interpreted as a castle.
            // TODO: test this
            if m.is_castle() && m.from_square() == from && m.castling_rook_squares().0 == to {
                return true;
            }

            false
        })
    }

    /// Make a move on the board. (See [`Self::push_uci`] for a more convenient
    /// interface.)
    ///
    /// Attempts to find a legal move matching the provided parameters. If one
    /// is found, the move is made on the board and is returned. If no legal
    /// moves match the criteria, [`None`] is returned.
    ///
    /// `promotion` should be `None` unless the move involves a pawn moving to
    /// the back rank, in which case it should be `Some` of a valid promotion
    /// piece type.
    pub fn push_move(
        &mut self,
        from: Square,
        to: Square,
        promotion: Option<PieceType>,
    ) -> Option<Move> {
        let the_move = self.get_move(from, to, promotion);

        if let Some(the_move) = the_move {
            self.push_move_unchecked(the_move);
        }

        the_move
    }

    /// Make a move on the board, using the move format in Universal Chess
    /// Interface (UCI) to specify the move.
    ///
    /// See [`UCIMove`] for more details about the format.
    ///
    /// Attempts to find a legal move matching the provided move. If one is
    /// found, the move is made on the board and is returned. If no legal moves
    /// match the criteria, [`None`] is returned.
    ///
    /// # Examples
    /// ```
    /// # use hardfiskur_core::board::Board;
    /// let mut board = Board::starting_position();
    /// assert!(board.push_uci("e2e4").is_some());
    /// assert!(board.push_uci("d7d5").is_some());
    /// assert!(board.push_uci("e1e2").is_some());
    ///
    /// // Promotion, but obviously not possible right now
    /// assert!(board.push_uci("a2a1q").is_none());
    /// ```
    pub fn push_uci(&mut self, uci: &str) -> Option<Move> {
        let UCIMove {
            from,
            to,
            promotion,
        } = uci.parse().ok()?;

        self.push_move(from, to, promotion)
    }

    /// Make a move on the board.
    ///
    /// Checks first if the move is legal. If it is, the move is made on the
    /// board and `true` is returned. Otherwise, `false` is returned.
    pub fn push_move_repr(&mut self, the_move: Move) -> bool {
        let legal_moves = self.legal_moves();

        if legal_moves.contains(&the_move) {
            self.push_move_unchecked(the_move);
            true
        } else {
            false
        }
    }

    /// Make a move on the board without checking its legality.
    ///
    /// Ensure that the move provided is legal, otherwise you will put the board
    /// into an invalid state.
    pub fn push_move_unchecked(&mut self, the_move: Move) {
        let unmake = self.make_move_unchecked(the_move);
        self.move_history.push(unmake);
    }

    /// Undo the most recently made move on the board.
    ///
    /// Does nothing if there are no moves in the move history. Returns the move
    /// that was undone if any.
    pub fn pop_move(&mut self) -> Option<Move> {
        let unmake_data = self.move_history.pop()?;
        self.unmake_move(unmake_data);
        unmake_data.the_move
    }

    pub fn repr(&self) -> &BoardRepr {
        &self.board
    }

    /// Returns the bitboard representing all pieces of the given color.
    pub fn get_bitboard_for_color(&self, color: Color) -> Bitboard {
        self.board[color]
    }

    /// Returns the bitboard representing all positions of the given piece.
    pub fn get_bitboard_for_piece(&self, piece: Piece) -> Bitboard {
        self.board[piece]
    }

    pub fn get_bitboard_for_piece_type(&self, piece_type: PieceType) -> Bitboard {
        self.board[piece_type.with_color(Color::White)]
            | self.board[piece_type.with_color(Color::Black)]
    }

    pub fn get_occupied_bitboard(&self) -> Bitboard {
        self.board[Color::White] | self.board[Color::Black]
    }

    /// Returns all positions of the given piece.
    pub fn find_piece(&self, piece: Piece) -> impl Iterator<Item = Square> {
        self.board[piece].squares()
    }

    /// Finds the king of the given color.
    ///
    /// This method assumes that the board is in a valid state (i.e. exactly 1
    /// king of each color). Panics if none or multiple kings are found.
    pub fn get_king(&self, color: Color) -> Square {
        let mut kings = self.board[Piece::king(color)].squares();
        let square = kings.next().expect("No kings found");
        match kings.next() {
            Some(_) => panic!("Multiple kings found"),
            None => square,
        }
    }

    /// Finds the king of the given color.
    ///
    /// No checking to see if the board is in a valid state is done.
    pub fn get_king_unchecked(&self, color: Color) -> Option<Square> {
        self.board[Piece::king(color)].to_square()
    }

    /// Checks if the current position is drawn by insufficient material.
    ///
    /// Only returns true in situations where no legal sequence of moves can
    /// lead to a checkmate, i.e. the following situations:
    /// * Both sides only have a king
    /// * One side has a king and a minor piece against a king
    /// * Both sides have a king and bishop, but the bishops are the same colour
    ///
    /// This is a reasonably fast operation (as it is just a bunch of bitboard
    /// operations), and so may be called in the engine evaluation function.
    /// However, you probably want better draw evaluation in your evaluation
    /// function than what this method provides.
    pub fn check_draw_by_insufficient_material(&self) -> bool {
        let major_pieces_and_pawns = self.board[Piece::WHITE_ROOK]
            | self.board[Piece::WHITE_QUEEN]
            | self.board[Piece::BLACK_ROOK]
            | self.board[Piece::BLACK_QUEEN]
            | self.board[Piece::WHITE_PAWN]
            | self.board[Piece::BLACK_PAWN];

        if major_pieces_and_pawns != Bitboard::EMPTY {
            return false;
        }

        // Only kings and minor pieces are left.

        let [knights, white_bishops, black_bishops] = [
            (self.board[Piece::WHITE_KNIGHT] | self.board[Piece::BLACK_KNIGHT]).pop_count(),
            self.board[Piece::WHITE_BISHOP].pop_count(),
            self.board[Piece::BLACK_BISHOP].pop_count(),
        ];
        let minor_piece_count = knights + white_bishops + black_bishops;

        // bare kings, or one side has a king plus a minor piece
        if minor_piece_count <= 1 {
            return true;
        }

        // Check for 2 bishops of the same color
        if minor_piece_count == 2 && white_bishops == 1 && black_bishops == 1 {
            let white_bishop = self.board[Piece::WHITE_BISHOP].to_square().unwrap();
            let black_bishop = self.board[Piece::BLACK_BISHOP].to_square().unwrap();

            if white_bishop.parity() == black_bishop.parity() {
                return true;
            }
        }

        false
    }

    /// Checks for draw by threefold repetition. Returns true if the current
    /// position has been seen at least 2 times in the game history.
    ///
    /// This is a relatively slow method that should not be used in the engine
    /// -- an approximation there such as checking the Zobrist hash should
    /// suffice. However, it is exact -- i.e. it accurately determines if the
    /// current exact position has occurred at least twice before, taking into
    /// account castling rights and en passant.
    pub fn check_draw_by_repetition(&self) -> bool {
        #[derive(Debug, Clone, Copy)]
        struct ChainListItem {
            from: Square,
            to: Square,
        }

        // Can only ever be a maxmimum of 24 pieces that can make reversible
        // moves.
        let mut chain_list = Vec::<ChainListItem>::with_capacity(24);
        let mut repetitions = 0;
        let mut earliest_repetition_found = 0;

        for (move_history_pos, unmake_data) in self.move_history.iter().enumerate().rev() {
            let m = match unmake_data.the_move {
                Some(m) => m,
                None => continue,
            };

            if !m.is_reversible() {
                break;
            }

            match chain_list.iter().position(|i| i.from == m.to_square()) {
                Some(i) => {
                    let item = &mut chain_list[i];
                    // Concat moves
                    item.from = m.from_square();

                    if item.from == item.to {
                        // Remove this from the chain list (since it's now a
                        // null entry anyway)
                        chain_list.swap_remove(i);

                        // Chain list being empty means every piece is back to
                        // its original position at this point.
                        // This is a repetition if the side to move is also the
                        // same...
                        if chain_list.is_empty() && m.piece().color() == self.to_move {
                            repetitions += 1;
                            earliest_repetition_found = move_history_pos;

                            // (The current position counts as a repetition,
                            // so we check for == 2 rather than == 3)
                            if repetitions >= 2 {
                                break;
                            }
                        }
                    }
                }
                None => {
                    debug_assert!(m.from_square() != m.to_square());
                    chain_list.push(ChainListItem {
                        from: m.from_square(),
                        to: m.to_square(),
                    });
                }
            }
        }

        if repetitions >= 2 {
            // Extremely pernicious edge cases -- although we see the same board
            // position 3 times, this could not be a true repetition because:
            // * The castling state is different in this position vs. the
            //   current one
            // * The repeated position resulted from a double pawn push, which
            //   could have been captured en passant, but no more in the current
            //   position

            let unmake_data = &self.move_history[earliest_repetition_found];

            if self.castling != unmake_data.castling {
                // Not a true repetition
                // (Since castling rights can only ever be removed, never added,
                // there couldn't be an earlier position where the castling bits
                // are the same)
                return false;
            }

            if unmake_data.en_passant.is_some() {
                // The initial repeated position was caused by a double pawn
                // push. This is especially annoying because it still counts as
                // a repetition if en passant wasn't possible, which could be
                // because:
                // * There isn't a pawn in position to actually capture en
                //   passant
                // * The pawn that could capture en passant is pinned
                // * By capturing en passant, removing the capturing and
                //   captured pawn exposes a horizontal check by a rook or a
                //   queen

                // Rather than jank together some new code for detecting this,
                // the move generation is already capable of handling all these
                // cases. So even though this calling the move generation again
                // is somewhat expensive this should be a pretty rare case that
                // it really shouldn't matter.

                let result = MoveGenerator::new(
                    &self.board,
                    self.to_move,
                    // use the en passant state of the first repetition
                    unmake_data.en_passant,
                    self.castling,
                    MoveGenFlags::GEN_CAPTURES,
                    // (Don't care about the moves)
                    &mut MoveVec::new(),
                )
                .legal_moves();
                if result.en_passant_possible {
                    // Not a true repetition
                    // (Since this position resulted from a double pawn push,
                    // it's an irreversible move and positions prior to this
                    // can't be a repetition)
                    return false;
                }
            }

            true
        } else {
            false
        }
    }

    // TODO: Document
    pub fn last_move(&self) -> Option<Move> {
        self.move_history
            .last()
            .and_then(|unmake_data| unmake_data.the_move)
    }

    pub fn attacked_by_pawn(&self, current_player: Color, square: Square) -> bool {
        let attacked_bb = match current_player {
            Color::White => move_gen::black_pawn_attacks(self.board[Piece::BLACK_PAWN]),
            Color::Black => move_gen::white_pawn_attacks(self.board[Piece::WHITE_PAWN]),
        };
        attacked_bb.get(square)
    }

    pub fn current_position_repeated_at_least(&self, times: u32) -> bool {
        let mut repetitions = 0;
        for (unmake_data, _) in self
            .move_history
            .iter()
            .rev()
            .zip(0..self.halfmove_clock)
            .skip(1)
            .step_by(2)
        {
            if unmake_data.zobrist_hash == self.zobrist_hash {
                repetitions += 1;

                if repetitions >= times {
                    return true;
                }
            }
        }

        false
    }

    fn non_board_hash(
        to_move: Color,
        castling: Castling,
        en_passant: Option<Square>,
    ) -> ZobristHash {
        ZobristHash::color(to_move)
            ^ ZobristHash::castling(castling)
            ^ ZobristHash::en_passant(en_passant)
    }

    fn castling_rights_removed(the_move: Move) -> Castling {
        let mut removed_rights = Castling::empty();

        if the_move.is_move_of(PieceType::King) {
            removed_rights |= match the_move.piece().color() {
                Color::White => Castling::WHITE,
                Color::Black => Castling::BLACK,
            };
        } else if the_move.is_move_of(PieceType::Rook) {
            removed_rights |= match the_move.from_square() {
                Square::WHITE_KINGSIDE_ROOK => Castling::WHITE_KINGSIDE,
                Square::WHITE_QUEENSIDE_ROOK => Castling::WHITE_QUEENSIDE,
                Square::BLACK_KINGSIDE_ROOK => Castling::BLACK_KINGSIDE,
                Square::BLACK_QUEENSIDE_ROOK => Castling::BLACK_QUEENSIDE,
                _ => Castling::empty(),
            };
        }

        if the_move.is_capture_of(PieceType::Rook) {
            removed_rights |= match the_move.to_square() {
                Square::WHITE_KINGSIDE_ROOK => Castling::WHITE_KINGSIDE,
                Square::WHITE_QUEENSIDE_ROOK => Castling::WHITE_QUEENSIDE,
                Square::BLACK_KINGSIDE_ROOK => Castling::BLACK_KINGSIDE,
                Square::BLACK_QUEENSIDE_ROOK => Castling::BLACK_QUEENSIDE,
                _ => Castling::empty(),
            };
        }

        removed_rights
    }

    fn make_move_unchecked(&mut self, the_move: Move) -> UnmakeData {
        self.to_move = self.to_move.flip();
        if self.to_move.is_white() {
            self.fullmoves += 1;
        }

        self.board.move_unchecked(the_move);

        // Update if the move broke any castling rights
        let prev_castling = self.castling;
        self.castling
            .remove(Self::castling_rights_removed(the_move));

        // Set the en passant square if applicable
        let prev_en_passant = self.en_passant.take();
        if the_move.is_double_pawn_push() {
            // Little trick -- due to our square representation, the square
            // inbetween two squares vertically is simply the average of the
            // start and end square
            let en_passant_square = (the_move.from_square().get() + the_move.to_square().get()) / 2;
            self.en_passant = Some(Square::from_u8_unchecked(en_passant_square));
        }

        let prev_halfmove_clock = self.halfmove_clock;
        if the_move.is_capture() || the_move.is_move_of(PieceType::Pawn) {
            self.halfmove_clock = 0;
        } else {
            self.halfmove_clock += 1;
        }

        let prev_zobrist_hash = self.zobrist_hash;
        self.zobrist_hash = self.board.zobrist_hash()
            ^ Self::non_board_hash(self.to_move, self.castling, self.en_passant);

        UnmakeData {
            the_move: Some(the_move),
            castling: prev_castling,
            en_passant: prev_en_passant,
            halfmove_clock: prev_halfmove_clock,
            zobrist_hash: prev_zobrist_hash,
        }
    }

    fn unmake_move(&mut self, unmake_data: UnmakeData) {
        let UnmakeData {
            the_move,
            castling,
            en_passant,
            halfmove_clock,
            zobrist_hash,
        } = unmake_data;

        self.to_move = self.to_move.flip();
        if self.to_move.is_black() {
            self.fullmoves -= 1;
        }

        if let Some(the_move) = the_move {
            self.board.move_unchecked(the_move);
        }

        self.castling = castling;
        self.en_passant = en_passant;
        self.halfmove_clock = halfmove_clock;
        self.zobrist_hash = zobrist_hash;
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::starting_position()
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.board)?;
        writeln!(f, "Move {} -- {:?} to move", self.fullmoves, self.to_move)?;

        write!(f, "Castling rights: {}", self.castling)?;
        if let Some(square) = self.en_passant {
            write!(f, "\tEn passant available on {}", square)?;
        }
        writeln!(f)?;

        write!(
            f,
            "Plies since last capture or pawn push: {}",
            self.halfmove_clock
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::test_utils::assert_in_any_order;

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn board_default_is_starting_position() {
        let default = Board::default();
        let start_pos = Board::starting_position();

        assert_eq!(default, start_pos);
    }

    #[test]
    fn board_display_representation() {
        let board =
            Board::try_parse_fen("rnbqkb1r/ppp1pppp/5n2/8/2pP4/5N2/PP2PPPP/RNBQKB1R w KQkq - 2 4")
                .unwrap();
        let expected = "+---+---+---+---+---+---+---+---+
|*r |*n |*b |*q |*k |*b | \u{00a0} |*r | 8
+---+---+---+---+---+---+---+---+
|*p |*p |*p | \u{00a0} |*p |*p |*p |*p | 7
+---+---+---+---+---+---+---+---+
| \u{00a0} | . | \u{00a0} | . | \u{00a0} |*n | \u{00a0} | . | 6
+---+---+---+---+---+---+---+---+
| . | \u{00a0} | . | \u{00a0} | . | \u{00a0} | . | \u{00a0} | 5
+---+---+---+---+---+---+---+---+
| \u{00a0} | . |*p | P | \u{00a0} | . | \u{00a0} | . | 4
+---+---+---+---+---+---+---+---+
| . | \u{00a0} | . | \u{00a0} | . | N | . | \u{00a0} | 3
+---+---+---+---+---+---+---+---+
| P | P | \u{00a0} | . | P | P | P | P | 2
+---+---+---+---+---+---+---+---+
| R | N | B | Q | K | B | . | R | 1
+---+---+---+---+---+---+---+---+
  a   b   c   d   e   f   g   h
Move 4 -- White to move
Castling rights: KQkq
Plies since last capture or pawn push: 2";

        assert_eq!(format!("{board}"), expected)
    }

    #[test]
    fn board_display_representation_with_en_passant() {
        let board =
            Board::try_parse_fen("rnbqkb1r/pp3ppp/3p4/3Pp3/3N4/2N5/PPP2PPP/R1BQKB1R w KQkq e6 0 7")
                .unwrap();
        let expected = "+---+---+---+---+---+---+---+---+
|*r |*n |*b |*q |*k |*b | \u{00a0} |*r | 8
+---+---+---+---+---+---+---+---+
|*p |*p | . | \u{00a0} | . |*p |*p |*p | 7
+---+---+---+---+---+---+---+---+
| \u{00a0} | . | \u{00a0} |*p | \u{00a0} | . | \u{00a0} | . | 6
+---+---+---+---+---+---+---+---+
| . | \u{00a0} | . | P |*p | \u{00a0} | . | \u{00a0} | 5
+---+---+---+---+---+---+---+---+
| \u{00a0} | . | \u{00a0} | N | \u{00a0} | . | \u{00a0} | . | 4
+---+---+---+---+---+---+---+---+
| . | \u{00a0} | N | \u{00a0} | . | \u{00a0} | . | \u{00a0} | 3
+---+---+---+---+---+---+---+---+
| P | P | P | . | \u{00a0} | P | P | P | 2
+---+---+---+---+---+---+---+---+
| R | \u{00a0} | B | Q | K | B | . | R | 1
+---+---+---+---+---+---+---+---+
  a   b   c   d   e   f   g   h
Move 7 -- White to move
Castling rights: KQkq\tEn passant available on e6
Plies since last capture or pawn push: 0";

        assert_eq!(format!("{board}"), expected);
    }

    #[test]
    fn board_pieces() {
        let board =
            Board::try_parse_fen("8/p7/1p1k1pp1/3b4/3p1PP1/3P4/P1P1K2N/8 w - - 0 1").unwrap();

        assert_in_any_order(
            board.pieces(),
            vec![
                (Piece::BLACK_PAWN, Square::A7),
                (Piece::BLACK_PAWN, Square::B6),
                (Piece::BLACK_KING, Square::D6),
                (Piece::BLACK_PAWN, Square::F6),
                (Piece::BLACK_PAWN, Square::G6),
                (Piece::BLACK_BISHOP, Square::D5),
                (Piece::BLACK_PAWN, Square::D4),
                (Piece::WHITE_PAWN, Square::F4),
                (Piece::WHITE_PAWN, Square::G4),
                (Piece::WHITE_PAWN, Square::D3),
                (Piece::WHITE_PAWN, Square::A2),
                (Piece::WHITE_PAWN, Square::C2),
                (Piece::WHITE_KING, Square::E2),
                (Piece::WHITE_KNIGHT, Square::H2),
            ],
        )
    }

    #[test]
    fn board_get_pieces() {
        let board =
            Board::try_parse_fen("8/p7/1p1k1pp1/3b4/3p1PP1/3P4/P1P1K2N/8 w - - 0 1").unwrap();

        assert_eq!(board.get_piece(Square::D6), Some(Piece::BLACK_KING));
        assert_eq!(board.get_piece(Square::D5), Some(Piece::BLACK_BISHOP));
        assert_eq!(board.get_piece(Square::H2), Some(Piece::WHITE_KNIGHT));
        assert_eq!(board.get_piece(Square::E1), None);
    }

    #[test]
    fn board_legal_moves() {
        let board = Board::try_parse_fen("4r1k1/8/8/8/8/8/6P1/4nKn1 w - - 0 1").unwrap();
        let (moves, result) = board.legal_moves_and_meta();

        assert_in_any_order(
            moves,
            vec![
                MoveBuilder::new(Square::F1, Square::F2, Piece::WHITE_KING).build(),
                MoveBuilder::new(Square::F1, Square::G1, Piece::WHITE_KING)
                    .captures(Piece::BLACK_KNIGHT)
                    .build(),
                MoveBuilder::new(Square::G2, Square::G3, Piece::WHITE_PAWN).build(),
                MoveBuilder::new(Square::G2, Square::G4, Piece::WHITE_PAWN)
                    .is_double_pawn_push()
                    .build(),
            ],
        );
        assert_eq!(result.checker_count, 0);
    }

    #[test]
    fn board_legal_moves_in_check() {
        let board = Board::try_parse_fen("5rk1/8/8/8/8/8/6R1/4nK2 w - - 0 1").unwrap();
        let (moves, result) = board.legal_moves_and_meta();

        assert_in_any_order(
            moves,
            vec![
                MoveBuilder::new(Square::F1, Square::E1, Piece::WHITE_KING)
                    .captures(Piece::BLACK_KNIGHT)
                    .build(),
                MoveBuilder::new(Square::F1, Square::E2, Piece::WHITE_KING).build(),
                MoveBuilder::new(Square::F1, Square::G1, Piece::WHITE_KING).build(),
                MoveBuilder::new(Square::G2, Square::F2, Piece::WHITE_ROOK).build(),
            ],
        );
        assert_eq!(result.checker_count, 1);
    }

    #[test]
    fn board_legal_moves_in_double_check() {
        let board = Board::try_parse_fen("5rk1/8/8/8/8/3b4/6R1/4NK2 w - - 0 1").unwrap();
        let (moves, result) = board.legal_moves_and_meta();

        assert_in_any_order(
            moves,
            vec![MoveBuilder::new(Square::F1, Square::G1, Piece::WHITE_KING).build()],
        );
        assert_eq!(result.checker_count, 2);
    }

    #[test]
    fn board_legal_moves_ex_only_pushes() {
        let board = Board::try_parse_fen("4r1k1/8/8/8/8/8/6P1/4nKn1 w - - 0 1").unwrap();
        let mut moves = MoveVec::new();
        let result = board.legal_moves_ex(MoveGenFlags::GEN_QUIET_MOVES, &mut moves);

        assert_in_any_order(
            moves,
            vec![
                MoveBuilder::new(Square::F1, Square::F2, Piece::WHITE_KING).build(),
                MoveBuilder::new(Square::G2, Square::G3, Piece::WHITE_PAWN).build(),
                MoveBuilder::new(Square::G2, Square::G4, Piece::WHITE_PAWN)
                    .is_double_pawn_push()
                    .build(),
            ],
        );
        assert_eq!(result.checker_count, 0);
    }

    #[test]
    fn board_legal_moves_ex_only_captures() {
        let board = Board::try_parse_fen("4r1k1/8/8/8/8/8/6P1/4nKn1 w - - 0 1").unwrap();
        let mut moves = MoveVec::new();
        let result = board.legal_moves_ex(MoveGenFlags::GEN_CAPTURES, &mut moves);

        assert_in_any_order(
            moves,
            vec![MoveBuilder::new(Square::F1, Square::G1, Piece::WHITE_KING)
                .captures(Piece::BLACK_KNIGHT)
                .build()],
        );
        assert_eq!(result.checker_count, 0);
    }

    fn assert_sequence_of_legal_moves(
        mut board: Board,
        ops: Vec<(&'static str, Box<dyn Fn(&Board)>)>,
    ) {
        let mut board_states = vec![board.clone()];

        for (i, (uci_move, asserter)) in ops.iter().enumerate() {
            assert!(
                board.push_uci(uci_move).is_some(),
                "failed on move {i}: {uci_move} is not a valid move"
            );
            asserter(&board);

            board_states.push(board.clone());
        }

        for (_, asserter) in ops.iter().rev() {
            assert_eq!(board, board_states.pop().unwrap());
            asserter(&board);

            assert!(board.pop_move().is_some());
        }

        assert_eq!(board, board_states.pop().unwrap());
    }

    #[test]
    fn board_push_invalid_move_returns_false() {
        let mut board = Board::starting_position();

        assert!(board.push_move(Square::E1, Square::E2, None).is_none());
    }

    #[test]
    fn board_push_invalid_uci_returns_none() {
        let mut board = Board::starting_position();

        assert!(board.push_uci("xxxx").is_none());
    }

    #[test]
    fn board_push_move_repr_returns_true_for_valid_move() {
        let mut board = Board::starting_position();
        let m = MoveBuilder::new(Square::E2, Square::E4, Piece::WHITE_PAWN)
            .is_double_pawn_push()
            .build();

        assert!(board.push_move_repr(m));
    }

    #[test]
    fn board_push_move_repr_returns_false_for_invalid_move() {
        let mut board = Board::starting_position();
        let m = MoveBuilder::new(Square::E1, Square::E2, Piece::WHITE_KING).build();

        assert!(!board.push_move_repr(m));
    }

    #[test]
    fn board_pop_moves_when_no_move_history_returns_none() {
        let mut board = Board::starting_position();

        assert_eq!(board.pop_move(), None);
    }

    #[test]
    fn board_push_and_pop_move() {
        let mut board = Board::starting_position();

        assert!(board.push_move(Square::E2, Square::E4, None).is_some());

        let popped_move = board.pop_move();
        assert_eq!(
            popped_move,
            Some(
                Move::builder(Square::E2, Square::E4, Piece::WHITE_PAWN)
                    .is_double_pawn_push()
                    .build()
            )
        );

        assert_eq!(board, Board::starting_position());
    }

    #[test]
    fn board_adjusts_to_move_and_fullmoves_correctly() {
        assert_sequence_of_legal_moves(
            Board::starting_position(),
            vec![
                (
                    "e2e4",
                    Box::new(|board| {
                        assert_eq!(board.to_move(), Color::Black);
                        assert_eq!(board.fullmoves(), 1);
                    }),
                ),
                (
                    "e7e5",
                    Box::new(|board| {
                        assert_eq!(board.to_move(), Color::White);
                        assert_eq!(board.fullmoves(), 2);
                    }),
                ),
                (
                    "g1f3",
                    Box::new(|board| {
                        assert_eq!(board.to_move(), Color::Black);
                        assert_eq!(board.fullmoves(), 2);
                    }),
                ),
            ],
        );
    }

    #[test]
    fn board_adjusts_castling_correctly_after_castle_and_king_move() {
        assert_sequence_of_legal_moves(
            Board::try_parse_fen("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1").unwrap(),
            vec![
                (
                    "e1g1",
                    Box::new(|board| assert_eq!(board.castling(), Castling::BLACK)),
                ),
                (
                    "e8e7",
                    Box::new(|board| assert_eq!(board.castling(), Castling::empty())),
                ),
            ],
        );
    }

    #[test]
    fn board_adjusts_castling_correctly_after_rooks_moved_or_captured() {
        assert_sequence_of_legal_moves(
            Board::try_parse_fen("r3k2r/8/8/8/8/6n1/8/R3K2R b KQkq - 0 1").unwrap(),
            vec![
                (
                    "g3h1",
                    Box::new(|board| {
                        assert_eq!(
                            board.castling(),
                            Castling::WHITE_QUEENSIDE | Castling::BLACK
                        )
                    }),
                ),
                (
                    "a1a8",
                    Box::new(|board| assert_eq!(board.castling(), Castling::BLACK_KINGSIDE)),
                ),
            ],
        );

        assert_sequence_of_legal_moves(
            Board::try_parse_fen("r3k2r/8/8/8/8/1n6/8/R3K2R b KQkq - 0 1").unwrap(),
            vec![
                (
                    "b3a1",
                    Box::new(|board| {
                        assert_eq!(board.castling(), Castling::WHITE_KINGSIDE | Castling::BLACK)
                    }),
                ),
                (
                    "h1h8",
                    Box::new(|board| assert_eq!(board.castling(), Castling::BLACK_QUEENSIDE)),
                ),
            ],
        );
    }

    #[test]
    fn board_updates_en_passant_correctly() {
        assert_sequence_of_legal_moves(
            Board::try_parse_fen("4k3/4p3/8/8/p1p2P2/8/1P4P1/4K3 w - - 0 1").unwrap(),
            vec![
                (
                    "f4f5",
                    Box::new(|board| assert_eq!(board.en_passant(), None)),
                ),
                (
                    "e7e5",
                    Box::new(|board| assert_eq!(board.en_passant(), Some(Square::E6))),
                ),
                (
                    "b2b4",
                    Box::new(|board| assert_eq!(board.en_passant(), Some(Square::B3))),
                ),
                (
                    "c4b3",
                    Box::new(|board| assert_eq!(board.en_passant(), None)),
                ),
            ],
        )
    }

    #[test]
    fn board_updates_halfmove_clock_correctly() {
        assert_sequence_of_legal_moves(
            Board::try_parse_fen("4k3/p7/2P4R/8/1r6/8/5b2/5K2 w - - 0 1").unwrap(),
            vec![
                (
                    "h6f6",
                    Box::new(|board| assert_eq!(board.halfmove_clock(), 1)),
                ),
                (
                    "b4b3",
                    Box::new(|board| assert_eq!(board.halfmove_clock(), 2)),
                ),
                (
                    "f6f2",
                    Box::new(|board| assert_eq!(board.halfmove_clock(), 0)),
                ),
                (
                    "b3b4",
                    Box::new(|board| assert_eq!(board.halfmove_clock(), 1)),
                ),
                (
                    "c6c7",
                    Box::new(|board| assert_eq!(board.halfmove_clock(), 0)),
                ),
                (
                    "b4b3",
                    Box::new(|board| assert_eq!(board.halfmove_clock(), 1)),
                ),
                (
                    "c7c8q",
                    Box::new(|board| assert_eq!(board.halfmove_clock(), 0)),
                ),
            ],
        )
    }

    #[test]
    fn board_reports_in_play_correctly() {
        let mut board = Board::starting_position();

        assert_eq!(board.state(), BoardState::InPlay { checkers: 0 });

        board.push_uci("e2e4").unwrap();

        assert_eq!(board.state(), BoardState::InPlay { checkers: 0 });
    }

    #[test]
    fn board_reports_checks_correctly() {
        let mut board = Board::try_parse_fen("4k3/3Q4/8/8/8/6b1/P4n2/4K3 b - - 0 1").unwrap();
        assert_eq!(board.state(), BoardState::InPlay { checkers: 1 });

        board.push_uci("e8d7").unwrap();
        assert_eq!(board.state(), BoardState::InPlay { checkers: 0 });

        board.push_uci("a2a4").unwrap();
        assert_eq!(board.state(), BoardState::InPlay { checkers: 0 });

        board.push_uci("f2d3").unwrap();
        assert_eq!(board.state(), BoardState::InPlay { checkers: 2 });
    }

    #[test]
    fn board_reports_checkmate_correctly() {
        let board = Board::try_parse_fen("8/8/8/8/8/4k3/8/4K2r w - - 0 1").unwrap();
        assert_eq!(board.state(), BoardState::Win(Color::Black));

        let board = Board::try_parse_fen("k7/8/NK6/8/8/8/8/7B b - - 0 1").unwrap();
        assert_eq!(board.state(), BoardState::Win(Color::White));

        let board = Board::try_parse_fen("6kn/6pp/4B2N/8/8/2K5/8/5R2 b - - 0 1").unwrap();
        assert_eq!(board.state(), BoardState::Win(Color::White));
    }

    #[test]
    fn board_reports_stalemate_correctly() {
        let board = Board::try_parse_fen("7k/8/4Q2K/8/8/8/8/8 b - - 0 1").unwrap();
        assert_eq!(board.state(), BoardState::Draw(DrawReason::Stalemate));

        let board = Board::try_parse_fen("r7/8/8/8/8/1n6/1P1k4/1K6 w - - 0 1").unwrap();
        assert_eq!(board.state(), BoardState::Draw(DrawReason::Stalemate));
    }

    #[test]
    fn board_reports_draw_by_insufficient_material() {
        let board = Board::try_parse_fen("8/8/8/8/3k4/8/8/1K6 w - - 0 1").unwrap();
        assert_eq!(
            board.state(),
            BoardState::Draw(DrawReason::InsufficientMaterial)
        );
    }

    #[test]
    fn board_reports_draw_by_repetition() {
        let mut board = Board::starting_position();

        for _ in 0..2 {
            assert_eq!(board.state(), BoardState::InPlay { checkers: 0 });
            board.push_uci("g1f3").unwrap();
            assert_eq!(board.state(), BoardState::InPlay { checkers: 0 });
            board.push_uci("b8c6").unwrap();
            assert_eq!(board.state(), BoardState::InPlay { checkers: 0 });
            board.push_uci("f3g1").unwrap();
            assert_eq!(board.state(), BoardState::InPlay { checkers: 0 });
            board.push_uci("c6b8").unwrap();
        }

        assert_eq!(
            board.state(),
            BoardState::Draw(DrawReason::ThreeFoldRepetition)
        );
    }

    #[test]
    fn board_reports_draw_by_fifty_move_rule() {
        let mut board = Board::try_parse_fen("k6K/p7/7P/8/8/8/8/Rr6 w - - 0 1").unwrap();
        board.push_uci("h6h7").unwrap();

        // Generate squares A1, B1, C1, ... H1, H2, G2, F2, ... B6, A6
        let mut rook_square_sequence = Vec::new();
        for rank in 0..6 {
            if rank % 2 == 0 {
                for file in 0..8 {
                    rook_square_sequence.push(Square::new(rank, file).unwrap());
                }
            } else {
                for file in (0..8).rev() {
                    rook_square_sequence.push(Square::new(rank, file).unwrap());
                }
            }
        }

        let rook_move_sequence = rook_square_sequence
            .iter()
            .cycle()
            .copied()
            .zip(rook_square_sequence.iter().cycle().skip(1).copied());

        let black_white_move_sequence = rook_move_sequence.clone().skip(1).zip(rook_move_sequence);

        // Just shuffle the rooks around a whole bunch, with
        // ... Rc1
        // Rb1 Rd1
        // Rc1 Re1 etc.
        let mut moves = 0;
        for (black_move, white_move) in black_white_move_sequence {
            assert_eq!(board.state(), BoardState::InPlay { checkers: 0 });
            board.push_move(black_move.0, black_move.1, None).unwrap();
            assert_eq!(board.state(), BoardState::InPlay { checkers: 0 });
            board.push_move(white_move.0, white_move.1, None).unwrap();

            moves += 1;
            if moves >= 50 {
                break;
            }
        }

        assert_eq!(board.state(), BoardState::Draw(DrawReason::FiftyMoveRule));
    }

    #[test]
    fn board_checks_draw_by_insufficient_material_positive_cases() {
        // Bare kings
        assert!(Board::try_parse_fen("4k3/8/8/8/8/8/8/4K3 w - - 0 1")
            .unwrap()
            .check_draw_by_insufficient_material());

        // Black has king + minor piece
        assert!(Board::try_parse_fen("4kn2/8/8/8/8/8/8/4K3 w - - 0 1")
            .unwrap()
            .check_draw_by_insufficient_material());

        // White has king + minor piece
        assert!(Board::try_parse_fen("4k3/8/8/2B5/8/8/8/4K3 w - - 0 1")
            .unwrap()
            .check_draw_by_insufficient_material());

        // Two bishops of the same color
        assert!(Board::try_parse_fen("4k3/8/8/8/3B1b2/8/8/4K3 w - - 0 1")
            .unwrap()
            .check_draw_by_insufficient_material());
    }

    #[test]
    fn board_checks_draw_by_insufficient_material_negative_cases() {
        // white has a pawn
        assert!(!Board::try_parse_fen("4k3/8/8/8/8/2P5/8/4K3 w - - 0 1")
            .unwrap()
            .check_draw_by_insufficient_material());

        // black has a queen
        assert!(!Board::try_parse_fen("4k3/8/6q1/8/8/8/8/4K3 w - - 0 1")
            .unwrap()
            .check_draw_by_insufficient_material());

        // white has a knight and black has a bishop
        assert!(!Board::try_parse_fen("4k3/8/3b4/8/8/8/8/4KN2 w - - 0 1")
            .unwrap()
            .check_draw_by_insufficient_material());

        // black has two minor pieces
        assert!(!Board::try_parse_fen("4k1n1/8/3b4/8/8/8/8/4K3 w - - 0 1")
            .unwrap()
            .check_draw_by_insufficient_material());

        // two bishops of opposite color
        assert!(!Board::try_parse_fen("4k3/8/3bB3/8/8/8/8/4K3 w - - 0 1")
            .unwrap()
            .check_draw_by_insufficient_material());
    }

    #[test]
    fn board_checks_draw_by_threefold_repetition_not_immediately_repeated_moves() {
        let mut board = Board::starting_position();

        let moves = [
            "e2e3", "e7e6", "c2c3", "c7c6", // Repetition 1
            "d1e2", "d8e7", "e2d1", "e7d8", // Repetition 2
            "d1g4", "d8a5", "g4a4", "a5g5", "a4d1", "g5d8", // Repetition 3
        ];

        for m in moves {
            assert_eq!(board.state(), BoardState::InPlay { checkers: 0 });
            board.push_uci(m).unwrap();
        }

        assert_eq!(
            board.state(),
            BoardState::Draw(DrawReason::ThreeFoldRepetition)
        );
    }

    #[test]
    fn board_checks_draw_by_repetition_loss_of_castling_rights() {
        let mut board = Board::starting_position();

        let moves = [
            "e2e4", "e7e5",
            // not actual 1st repetition due to both sides still having
            // castling rights
            "e1e2", "e8e7", // castling rights lost, repetition 1
            "e2e1", "e7e8", // (pseudo-repetition 2)
            "e1e2", "e8e7", // repetition 2
            "e2e1", "e7e8", // (pseudo-repetition 3)
            "e1e2", "e8e7", // repetition 3
        ];

        for m in moves {
            assert_eq!(board.state(), BoardState::InPlay { checkers: 0 });
            board.push_uci(m).unwrap();
        }

        assert_eq!(
            board.state(),
            BoardState::Draw(DrawReason::ThreeFoldRepetition)
        );
    }

    #[test]
    fn board_checks_draw_by_repetition_loss_of_en_passant_possibility() {
        let mut board = Board::starting_position();

        let moves = [
            "e2e4", "e7e6", "e4e5", "d7d5",
            // not first repetition due to en passant available for white's
            // e-pawn
            "g1f3", // Reptition 1
            "g8f6", "f3g1", "f6g8", // (pseudo-repetition 2)
            "g1f3", // Repetition 2
            "g8f6", "f3g1", "f6g8", // (pseudo-repetition 3)
            "g1f3", // Repetition 3
        ];

        for m in moves {
            assert_eq!(board.state(), BoardState::InPlay { checkers: 0 });
            board.push_uci(m).unwrap();
        }

        assert_eq!(
            board.state(),
            BoardState::Draw(DrawReason::ThreeFoldRepetition)
        );
    }

    #[test]
    fn board_checks_draw_by_repetition_edge_case_en_passant_not_possible_due_to_pinned_pawn() {
        let mut board =
            Board::try_parse_fen("rn1qkbnr/pppppppp/5b2/4P3/8/2K5/PPPP1PPP/RNBQ1BNR b kq - 0 1")
                .unwrap();

        let moves = [
            "d7d5",
            // repetition 1, en passant is "available" but not possible because
            // the en passant pin is actually pinned.
            "g1f3", "b8c6", "f3g1", "c6b8", // repetition 2
            "g1f3", "b8c6", "f3g1", "c6b8", // repetition 3
        ];

        for m in moves {
            assert_eq!(board.state(), BoardState::InPlay { checkers: 0 });
            board.push_uci(m).unwrap();
        }

        assert_eq!(
            board.state(),
            BoardState::Draw(DrawReason::ThreeFoldRepetition)
        );
    }

    #[test]
    fn board_checks_draw_by_repetition_edge_case_en_passant_not_possible_due_to_horizontal_pin() {
        let mut board =
            Board::try_parse_fen("rnb1kbnr/pppppppp/8/1K2P2q/8/8/PPPP1PPP/RNBQ1BNR b kq - 0 1")
                .unwrap();

        let moves = [
            "d7d5",
            // repetition 1, en passant is "available" but not possible because
            // the en passant pin is actually pinned.
            "g1f3", "b8c6", "f3g1", "c6b8", // repetition 2
            "g1f3", "b8c6", "f3g1", "c6b8", // repetition 3
        ];

        for m in moves {
            assert_eq!(board.state(), BoardState::InPlay { checkers: 0 });
            board.push_uci(m).unwrap();
        }

        assert_eq!(
            board.state(),
            BoardState::Draw(DrawReason::ThreeFoldRepetition)
        );
    }
}
