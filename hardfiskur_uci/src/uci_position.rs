use std::fmt::Display;

use hardfiskur_core::board::UCIMove;
use thiserror::Error;

use crate::parse_utils::{join_tokens, TokenSlice};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UCIPosition {
    pub base: UCIPositionBase,
    pub moves: Vec<UCIMove>,
}

impl Display for UCIPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.base)?;

        if !self.moves.is_empty() {
            write!(f, " moves")?;

            for m in self.moves.iter() {
                write!(f, " {m}")?;
            }
        }

        Ok(())
    }
}

impl UCIPosition {
    pub fn parse(tokens: TokenSlice) -> Result<(TokenSlice, Self), ParseUCIPositionError> {
        let (tokens, base) = UCIPositionBase::parse(tokens)?;

        let (tokens, moves) = match tokens {
            [("moves", _), rest @ ..] => {
                let mut moves = Vec::<UCIMove>::with_capacity(rest.len());
                for (m, _) in rest {
                    if let Ok(m) = m.parse() {
                        moves.push(m);
                    }
                }
                ([].as_slice(), moves)
            }
            rest => (rest, Vec::new()),
        };

        Ok((tokens, Self { base, moves }))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UCIPositionBase {
    StartPos,
    Fen(String),
}

impl UCIPositionBase {
    pub fn parse(tokens: TokenSlice) -> Result<(TokenSlice, Self), ParseUCIPositionError> {
        match tokens {
            [("startpos", _), rest @ ..] => Ok((rest, Self::StartPos)),
            [("fen", _), board, color, castling, en_passant, halfmove_clock, fullmoves, rest @ ..] => {
                Ok((
                    rest,
                    Self::Fen(join_tokens(&[
                        *board,
                        *color,
                        *castling,
                        *en_passant,
                        *halfmove_clock,
                        *fullmoves,
                    ])),
                ))
            }
            _ => Err(ParseUCIPositionError),
        }
    }
}

impl Display for UCIPositionBase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UCIPositionBase::StartPos => write!(f, "startpos"),
            UCIPositionBase::Fen(fen) => write!(f, "fen {fen}"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("Error parsing UCI position string")]
pub struct ParseUCIPositionError;
