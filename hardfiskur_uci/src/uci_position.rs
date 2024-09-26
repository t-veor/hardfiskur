use std::fmt::Display;

use hardfiskur_core::board::UCIMove;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UCIPositionBase {
    StartPos,
    Fen(String),
}

impl Display for UCIPositionBase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UCIPositionBase::StartPos => write!(f, "startpos"),
            UCIPositionBase::Fen(fen) => write!(f, "fen {fen}"),
        }
    }
}
