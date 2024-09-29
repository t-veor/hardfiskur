use std::{
    fmt::{Display, Write},
    str::FromStr,
};

use thiserror::Error;

use super::{Move, Piece, PieceType, Square};

/// Utility type representing a move as used in the Universal Chess Interface
/// (UCI).
///
/// This type is intended for parsing the move format used in UCI, which simply
/// specifies the start and end squares as well as optional promotion. Some
/// examples are:
///
/// * `e2e4`
/// * `e7e5`
/// * `e1g1` (white short castling)
/// * `e7e8q` (for promotion)
///
/// This is sometimes called long algebraic notation, but long algebraic
/// notation may have additional information than required by UCI, e.g. the
/// piece being moved, captures etc.
///
/// ```
/// # use hardfiskur_core::board::{UCIMove, Square};
/// assert_eq!(
///     "e2e4".parse(),
///     Ok(UCIMove {
///         from: Square::E2,
///         to: Square::E4,
///         promotion: None,
///     })
/// );
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UCIMove {
    pub from: Square,
    pub to: Square,
    pub promotion: Option<PieceType>,
}

impl Display for UCIMove {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.from.fmt(f)?;
        self.to.fmt(f)?;
        if let Some(promotion) = self.promotion {
            f.write_char(promotion.as_lowercase_char())?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ParseUCIMoveError {
    #[error("Expected 4 or 5 characters")]
    IncorrectLength,
    #[error("Invalid square {0}")]
    InvalidSquare(String),
    #[error("Invalid promo target")]
    InvalidPromoTarget(char),
}

impl FromStr for UCIMove {
    type Err = ParseUCIMoveError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let chars = s.chars().collect::<Vec<_>>();
        if chars.len() != 4 && chars.len() != 5 {
            return Err(ParseUCIMoveError::IncorrectLength);
        }

        let from_str = String::from_iter(&chars[0..2]);
        let from = from_str
            .parse()
            .map_err(|_| ParseUCIMoveError::InvalidSquare(from_str))?;

        let to_str = String::from_iter(&chars[2..4]);
        let to = to_str
            .parse()
            .map_err(|_| ParseUCIMoveError::InvalidSquare(to_str))?;

        let promotion = match chars.get(4) {
            Some(&c) => Some(
                Piece::try_from_fen_char(c)
                    .ok_or(ParseUCIMoveError::InvalidPromoTarget(c))?
                    .piece_type(),
            ),
            None => None,
        };

        Ok(Self {
            from,
            to,
            promotion,
        })
    }
}

impl From<Move> for UCIMove {
    fn from(value: Move) -> Self {
        Self {
            from: value.from_square(),
            to: value.to_square(),
            promotion: value.promotion().map(|p| p.piece_type()),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn parse_normal_move() {
        assert_eq!(
            "e2e4".parse(),
            Ok(UCIMove {
                from: Square::E2,
                to: Square::E4,
                promotion: None,
            })
        );

        assert_eq!(
            "b8c6".parse(),
            Ok(UCIMove {
                from: Square::B8,
                to: Square::C6,
                promotion: None,
            })
        );
    }

    #[test]
    fn parse_promotion() {
        assert_eq!(
            "e7e8q".parse(),
            Ok(UCIMove {
                from: Square::E7,
                to: Square::E8,
                promotion: Some(PieceType::Queen)
            })
        );

        assert_eq!(
            "a2a1b".parse(),
            Ok(UCIMove {
                from: Square::A2,
                to: Square::A1,
                promotion: Some(PieceType::Bishop),
            })
        );

        assert_eq!(
            "a2a1r".parse(),
            Ok(UCIMove {
                from: Square::A2,
                to: Square::A1,
                promotion: Some(PieceType::Rook),
            })
        );

        assert_eq!(
            "a2a1n".parse(),
            Ok(UCIMove {
                from: Square::A2,
                to: Square::A1,
                promotion: Some(PieceType::Knight),
            })
        );
    }

    #[test]
    fn parse_invalid_cases() {
        assert_eq!(
            UCIMove::from_str(""),
            Err(ParseUCIMoveError::IncorrectLength)
        );
        assert_eq!(
            UCIMove::from_str("e7e8qq"),
            Err(ParseUCIMoveError::IncorrectLength)
        );

        assert_eq!(
            UCIMove::from_str("a9e4"),
            Err(ParseUCIMoveError::InvalidSquare("a9".to_string())),
        );
        assert_eq!(
            UCIMove::from_str("a1xx"),
            Err(ParseUCIMoveError::InvalidSquare("xx".to_string())),
        );

        assert_eq!(
            UCIMove::from_str("e7e8x"),
            Err(ParseUCIMoveError::InvalidPromoTarget('x'))
        );
    }

    #[test]
    fn display_impl() {
        assert_eq!(format!("{}", UCIMove::from_str("e2e4").unwrap()), "e2e4");
        assert_eq!(format!("{}", UCIMove::from_str("b8c6").unwrap()), "b8c6");
        assert_eq!(format!("{}", UCIMove::from_str("e7e8q").unwrap()), "e7e8q");
    }
}
