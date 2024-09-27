use std::fmt::Display;

use hardfiskur_core::board::UCIMove;
use nom::{
    branch::alt,
    combinator::{opt, value},
    multi::{count, many0},
    sequence::preceded,
    IResult,
};
use thiserror::Error;

use crate::parse_utils::{parser_uci_move, token, token_tag};

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
    pub fn parser(input: &str) -> IResult<&str, Self> {
        let (input, base) = UCIPositionBase::parser(input)?;
        let (input, moves) = opt(preceded(token_tag("moves"), many0(parser_uci_move)))(input)?;

        Ok((
            input,
            Self {
                base,
                moves: moves.unwrap_or_default(),
            },
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UCIPositionBase {
    StartPos,
    Fen(String),
}

impl UCIPositionBase {
    pub fn parser(input: &str) -> IResult<&str, Self> {
        alt((
            value(Self::StartPos, token_tag("startpos")),
            Self::parser_fen,
        ))(input)
    }

    fn parser_fen(input: &str) -> IResult<&str, Self> {
        let (input, _) = token_tag("fen")(input)?;
        let (input, fen_parts) = count(token, 6)(input)?;

        Ok((input, Self::Fen(fen_parts.join(" "))))
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
