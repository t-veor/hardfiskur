use std::fmt::{Display, Write};

use bitflags::bitflags;

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
    /// Note these flags do not take into account temporary reasons for which a
    /// castle may not be permitted, e.g. there are pieces between the king and
    /// the corresponding rook, the king is in check or will move through or
    /// land in check, etc.
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

impl Default for Castling {
    fn default() -> Self {
        Self::all()
    }
}

impl Display for Castling {
    /// Returns the castling state as the 3rd field in a FEN string.
    ///
    /// See [`Self::as_fen_str`].
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_empty() {
            f.write_char('-')
        } else {
            if self.contains(Self::WHITE_KINGSIDE) {
                f.write_char('K')?;
            }
            if self.contains(Self::WHITE_QUEENSIDE) {
                f.write_char('Q')?;
            }
            if self.contains(Self::BLACK_KINGSIDE) {
                f.write_char('k')?;
            }
            if self.contains(Self::BLACK_QUEENSIDE) {
                f.write_char('q')?;
            }
            Ok(())
        }
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
        format!("{self}")
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
