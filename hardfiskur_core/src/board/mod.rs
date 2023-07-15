//! Structs and functions related to to the board representation.

mod bitboard;
mod board_repr;
mod fen;
mod move_repr;
mod piece;
mod square;

use bitflags::bitflags;

pub use bitboard::Bitboard;
pub use board_repr::BoardRepr;
pub use fen::FenParseError;
pub use move_repr::{Move, MoveFlags};
pub use piece::{Color, Piece, PieceType};
pub use square::Square;

use crate::move_gen::{MoveGenFlags, MoveGenResult, MoveGenerator, MoveVec};

pub const STARTING_POSITION_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

bitflags! {
    /// Represents which directions castling moves can still be played for
    /// both players.
    ///
    /// Castling is allowed if the king has not moved and the rook with which to
    /// castle has not moved (and some rules about whether the king is in check
    /// and whether any squares the king will move through or land on are
    /// attacked). Thus, these flags store whether castling is still allowed
    /// given the history of the game with the kingside or queenside rook.
    ///
    /// For example, after the white king makes a move, both the
    /// [`WHITE_KINGSIDE`](Self::WHITE_KINGSIDE) and
    /// [`WHITE_QUEENSIDE`](Self::WHITE_QUEENSIDE) flags will be set to 0 as
    /// castling is no longer allowed for the white king after it moves.
    /// However, if the black queenside rook makes a move, only
    /// [`BLACK_QUEENSIDE`](SELF::BLACK_QUEENSIDE) will be unset. This is
    /// because kingside castling is still possible for black if the black king
    /// and kingside rook have not yet moved.
    ///
    /// Note these flags do not take into account if there are any pieces
    /// between the king and the corresponding rook, whether the king is in
    /// check or whether the king passes through or lands on an attacked square.
    /// This will need to be checked during move generation.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Castling: u8 {
        /// White is allowed to castle kingside.
        const WHITE_KINGSIDE  = 0b0001;
        /// White is allowed to castline queenside.
        const WHITE_QUEENSIDE = 0b0010;
        /// Black is allowed to castle kingside.
        const BLACK_KINGSIDE  = 0b0100;
        /// Black is allowed to castle queenside.
        const BLACK_QUEENSIDE = 0b1000;

        const WHITE = Self::WHITE_KINGSIDE.bits() | Self::WHITE_QUEENSIDE.bits();
        const BLACK = Self::BLACK_KINGSIDE.bits() | Self::BLACK_QUEENSIDE.bits();
        const KINGSIDE = Self::WHITE_KINGSIDE.bits() | Self::BLACK_KINGSIDE.bits();
        const QUEENSIDE = Self::WHITE_QUEENSIDE.bits() | Self::BLACK_QUEENSIDE.bits();
    }
}

impl Castling {
    /// Returns the castling state as the 3rd field in [Forsyth-Edwards
    /// Notation](https://en.wikipedia.org/wiki/Forsyth%E2%80%93Edwards_Notation).
    ///
    /// If neither side can castle, returns `-`. Otherwise, returns a string
    /// that contains `K` if white can castle kingside, 'Q' if white can castle
    /// queenside, 'k' if black can castle kingside, and 'q' if black can castle
    /// queenside.
    pub fn as_fen_str(self) -> String {
        if self.is_empty() {
            "-".to_owned()
        } else {
            let mut result = String::with_capacity(4);
            if self.contains(Self::WHITE_KINGSIDE) {
                result.push('K');
            }
            if self.contains(Self::WHITE_QUEENSIDE) {
                result.push('Q');
            }
            if self.contains(Self::BLACK_KINGSIDE) {
                result.push('k');
            }
            if self.contains(Self::BLACK_QUEENSIDE) {
                result.push('q');
            }
            result
        }
    }
}

/// State of play for the board.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoardState {
    /// The player to move has legal moves available, and the game is not drawn.
    InPlay,
    /// The game is drawn.
    Draw,
    /// The game is over with a win for the specified player.
    Win(Color),
    /// The board is in an invalid state -- e.g. a king can be captured, there
    /// are no kings/too many kings, etc.
    Invalid,
}

#[derive(Debug, Clone, Copy)]
struct UnmakeData {
    the_move: Option<Move>,
    castling: Castling,
    en_passant: Option<Square>,
    halfmove_clock: u32,
    // TODO
    // zobrist_hash: u64,
}

#[derive(Debug, Clone)]
pub struct Board {
    board: BoardRepr,
    to_move: Color,
    castling: Castling,
    en_passant: Option<Square>,
    halfmove_clock: u32,
    fullmoves: u32,

    move_history: Vec<UnmakeData>,
}

impl Board {
    pub fn new(
        board: &[Option<Piece>],
        to_move: Color,
        castling: Castling,
        en_passant: Option<Square>,
        halfmove_clock: u32,
        fullmoves: u32,
    ) -> Self {
        let board = BoardRepr::new(board);

        // TODO: calculate Zobrist hash

        Self {
            board,
            to_move,
            castling,
            en_passant,
            halfmove_clock,
            fullmoves,

            move_history: Vec::new(),
        }
    }

    pub fn starting_position() -> Self {
        Self::try_parse_fen(STARTING_POSITION_FEN).unwrap()
    }

    pub fn to_move(&self) -> Color {
        self.to_move
    }

    pub fn pieces(&self) -> impl Iterator<Item = (Piece, Square)> + '_ {
        self.board.pieces()
    }

    pub fn get_piece(&self, square: Square) -> Option<Piece> {
        self.board.piece_at(square)
    }

    pub fn legal_moves(&self) -> (MoveVec, MoveGenResult) {
        let mut moves = MoveVec::new();
        let result = self.legal_moves_ex(MoveGenFlags::default(), &mut moves);
        (moves, result)
    }

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

    pub fn push_move(&mut self, from: Square, to: Square, promotion: Option<PieceType>) -> bool {
        let legal_moves = self.legal_moves().0;

        let the_move = legal_moves.into_iter().find(|m| {
            m.from_square() == from
                && m.to_square() == to
                && m.promotion().map(|piece| piece.piece_type()) == promotion
        });

        match the_move {
            Some(the_move) => {
                self.push_move_unchecked(the_move);
                true
            }
            None => false,
        }
    }

    pub fn push_move_unchecked(&mut self, the_move: Move) {
        let unmake = self.make_move_unchecked(the_move);
        self.move_history.push(unmake);
    }

    pub fn pop_move(&mut self) -> Option<Move> {
        let unmake_data = self.move_history.pop()?;
        self.unmake_move(unmake_data);
        unmake_data.the_move
    }

    fn castling_rights_removed(&self, the_move: Move) -> Option<Castling> {
        if the_move.is_move_of(PieceType::King) {
            Some(match the_move.piece().color() {
                Color::White => Castling::WHITE,
                Color::Black => Castling::BLACK,
            })
        } else if the_move.is_move_of(PieceType::Rook) {
            match the_move.from_square() {
                Square::WHITE_KINGSIDE_ROOK => Some(Castling::WHITE_KINGSIDE),
                Square::WHITE_QUEENSIDE_ROOK => Some(Castling::WHITE_QUEENSIDE),
                Square::BLACK_KINGSIDE_ROOK => Some(Castling::BLACK_KINGSIDE),
                Square::BLACK_QUEENSIDE_ROOK => Some(Castling::BLACK_QUEENSIDE),
                _ => None,
            }
        } else if the_move.is_capture_of(PieceType::Rook) {
            match the_move.to_square() {
                Square::WHITE_KINGSIDE_ROOK => Some(Castling::WHITE_KINGSIDE),
                Square::WHITE_QUEENSIDE_ROOK => Some(Castling::WHITE_QUEENSIDE),
                Square::BLACK_KINGSIDE_ROOK => Some(Castling::BLACK_KINGSIDE),
                Square::BLACK_QUEENSIDE_ROOK => Some(Castling::BLACK_QUEENSIDE),
                _ => None,
            }
        } else {
            None
        }
    }

    fn make_move_unchecked(&mut self, the_move: Move) -> UnmakeData {
        self.to_move = self.to_move.flip();
        if self.to_move.is_white() {
            self.fullmoves += 1;
        }

        self.board.move_unchecked(the_move);

        // Update if the move broke any castling rights
        let prev_castling = self.castling;
        if let Some(remove_rights) = self.castling_rights_removed(the_move) {
            self.castling.remove(remove_rights);
        }

        // Set the en passant square if applicable
        let prev_en_passant = self.en_passant;
        if the_move.is_double_pawn_push() {
            // Little trick -- due to our square representation, the square
            // inbetween two squares vertically is simply the average of the
            // start and end square
            let en_passant_square = (the_move.from_square().get() + the_move.to_square().get()) / 2;
            self.en_passant = Some(Square::from_u8_unchecked(en_passant_square));
        } else {
            self.en_passant = None;
        }

        let prev_halfmove_clock = self.halfmove_clock;
        if the_move.is_capture() || the_move.is_move_of(PieceType::Pawn) {
            self.halfmove_clock = 0;
        } else {
            self.halfmove_clock += 1;
        }

        // TODO: recalc zobrist hash and repetition table

        UnmakeData {
            the_move: Some(the_move),
            castling: prev_castling,
            en_passant: prev_en_passant,
            halfmove_clock: prev_halfmove_clock,
        }
    }

    fn unmake_move(&mut self, unmake_data: UnmakeData) {
        let UnmakeData {
            the_move,
            castling,
            en_passant,
            halfmove_clock,
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
        self.halfmove_clock = self.halfmove_clock;

        // TODO: reset zobrist hash and repetition table
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::starting_position()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn castling_as_fen_str() {
        assert_eq!(Castling::empty().as_fen_str(), "-");
        assert_eq!(Castling::WHITE_KINGSIDE.as_fen_str(), "K");
        assert_eq!(Castling::WHITE_QUEENSIDE.as_fen_str(), "Q");
        assert_eq!(Castling::BLACK_KINGSIDE.as_fen_str(), "k");
        assert_eq!(Castling::BLACK_QUEENSIDE.as_fen_str(), "q");

        assert_eq!(Castling::WHITE.as_fen_str(), "KQ");
        assert_eq!(Castling::BLACK.as_fen_str(), "kq");
        assert_eq!(Castling::KINGSIDE.as_fen_str(), "Kk");
        assert_eq!(Castling::QUEENSIDE.as_fen_str(), "Qq");

        assert_eq!(
            (Castling::WHITE_KINGSIDE | Castling::BLACK_QUEENSIDE).as_fen_str(),
            "Kq"
        );
        assert_eq!(
            (Castling::BLACK_KINGSIDE | Castling::WHITE_QUEENSIDE).as_fen_str(),
            "Qk"
        );

        assert_eq!(
            Castling::all()
                .difference(Castling::WHITE_KINGSIDE)
                .as_fen_str(),
            "Qkq"
        );
        assert_eq!(Castling::all().as_fen_str(), "KQkq");
    }
}
