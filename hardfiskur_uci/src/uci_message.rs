use std::{fmt::Display, str::FromStr};

use hardfiskur_core::board::UCIMove;
use thiserror::Error;

use crate::{
    parsing, uci_info::UCIInfo, uci_option_config::UCIOptionConfig, uci_position::UCIPosition,
    uci_search_control::UCISearchControl, uci_time_control::UCITimeControl,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UCIMessage {
    /// `uci`
    UCI,

    /// `debug [ on | off ]`
    Debug(bool),

    /// `isready`
    IsReady,

    /// `setoption name <id> [value <x>]`
    SetOption { name: String, value: Option<String> },

    /// `register later`
    /// `register name <name> code <code>`
    Register {
        later: bool,
        name: Option<String>,
        code: Option<String>,
    },

    /// `ucinewgame`
    UCINewGame,

    /// `position ...`
    Position(UCIPosition),

    /// `go ...`
    Go {
        time_control: Option<UCITimeControl>,
        search_control: Option<UCISearchControl>,
    },

    /// `stop`
    Stop,

    /// `ponderhit`
    PonderHit,

    /// `quit`
    Quit,

    /// `id [name <name>] [author <author>]`
    Id {
        name: Option<String>,
        author: Option<String>,
    },

    /// `uciok`
    UCIOk,

    /// `readyok`
    ReadyOk,

    /// `bestmove <best_move> [ponder <ponder_move>]`
    BestMove {
        best_move: UCIMove,
        ponder: Option<UCIMove>,
    },

    /// `copyprotection [checking | ok | error]`
    CopyProtection(ProtectionState),

    /// `registration [checking | ok | error]`
    Registration(ProtectionState),

    /// `info ...`
    Info(UCIInfo),

    /// `option ...`
    Option(UCIOptionConfig),

    // Custom commands for debugging
    /// `d`
    /// Print an ASCII representation of the current position, FEN string, and Zobrist hash.
    D,

    /// `ttentry`
    /// Print the transposition table entry for this position, if any.
    TTEntry,

    /// `[makemove | m] [move]`
    /// Makes the move provided on the current position.
    /// If move is not provided, makes the best move according to the transposition table if found.
    MakeMove(Option<UCIMove>),

    /// `[undomove | u]`
    /// Undo the last move on the current position.
    UndoMove,

    /// `pv`
    /// Get the principal variation from the transposition table.
    GetPV,

    /// `eval`
    /// Runs the evaluation function on this position and returns the result.
    /// Does not take into account quiescence, checkmates, draws etc.
    Eval,
}

impl UCIMessage {
    pub fn id_name(name: &str) -> Self {
        Self::Id {
            name: Some(name.to_string()),
            author: None,
        }
    }

    pub fn id_author(author: &str) -> Self {
        Self::Id {
            name: None,
            author: Some(author.to_string()),
        }
    }

    pub fn best_move(best_move: UCIMove) -> Self {
        Self::BestMove {
            best_move,
            ponder: None,
        }
    }
}

impl FromStr for UCIMessage {
    type Err = ParseUCIMessageError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match parsing::uci_message(s.trim()) {
            Ok((_remaining, message)) => Ok(message),
            Err(_) => Err(ParseUCIMessageError),
        }
    }
}

impl Display for UCIMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UCIMessage::UCI => write!(f, "uci"),

            UCIMessage::Debug(on) => {
                write!(f, "debug {}", if *on { "on" } else { "off" })
            }

            UCIMessage::IsReady => write!(f, "isready"),

            UCIMessage::SetOption { name, value } => {
                write!(f, "setoption name {name}")?;
                if let Some(value) = value {
                    write!(
                        f,
                        " value {}",
                        if value.is_empty() { "<empty>" } else { value }
                    )?;
                }
                Ok(())
            }

            UCIMessage::Register { later, name, code } => {
                if *later {
                    write!(f, "register later")
                } else {
                    write!(f, "register")?;
                    if let Some(name) = name {
                        write!(f, " name {name}")?;
                    }
                    if let Some(code) = code {
                        write!(f, " code {code}")?;
                    }
                    Ok(())
                }
            }

            UCIMessage::UCINewGame => write!(f, "ucinewgame"),

            UCIMessage::Position(pos) => write!(f, "position {pos}"),

            UCIMessage::Go {
                time_control,
                search_control,
            } => {
                write!(f, "go")?;
                if let Some(time_control) = time_control {
                    write!(f, " {time_control}")?;
                }
                if let Some(search_control) = search_control {
                    write!(f, " {search_control}")?;
                }
                Ok(())
            }

            UCIMessage::Stop => write!(f, "stop"),

            UCIMessage::PonderHit => write!(f, "ponderhit"),

            UCIMessage::Quit => write!(f, "quit"),

            UCIMessage::Id { name, author } => {
                write!(f, "id")?;
                if let Some(name) = name {
                    write!(f, " name {name}")?;
                }
                if let Some(author) = author {
                    write!(f, " author {author}")?;
                }
                Ok(())
            }

            UCIMessage::UCIOk => write!(f, "uciok"),

            UCIMessage::ReadyOk => write!(f, "readyok"),

            UCIMessage::BestMove { best_move, ponder } => {
                write!(f, "bestmove {best_move}")?;
                if let Some(ponder) = ponder {
                    write!(f, " ponder {ponder}")?;
                }
                Ok(())
            }

            UCIMessage::CopyProtection(protection_state) => {
                write!(f, "copyprotection {protection_state}")
            }

            UCIMessage::Registration(protection_state) => {
                write!(f, "registration {protection_state}")
            }

            UCIMessage::Info(info) => write!(f, "info {info}"),

            UCIMessage::Option(option_config) => write!(f, "option {option_config}"),

            UCIMessage::D => write!(f, "d"),
            UCIMessage::TTEntry => write!(f, "ttentry"),
            UCIMessage::MakeMove(m) => match m {
                Some(m) => write!(f, "makemove {m}"),
                None => write!(f, "makemove"),
            },
            UCIMessage::UndoMove => write!(f, "undomove"),
            UCIMessage::GetPV => write!(f, "getpv"),
            UCIMessage::Eval => write!(f, "eval"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtectionState {
    Checking,
    Ok,
    Error,
}

impl Display for ProtectionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            ProtectionState::Checking => "checking",
            ProtectionState::Ok => "ok",
            ProtectionState::Error => "error",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("Error parsing UCI message")]
pub struct ParseUCIMessageError;
