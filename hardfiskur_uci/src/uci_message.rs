use std::{fmt::Display, str::FromStr, time::Duration};

use hardfiskur_core::board::UCIMove;
use thiserror::Error;

use crate::{
    parse_utils::{parse_string_option, split_tokens, try_parse_many, try_parse_next, TokenSlice},
    uci_info::UCIInfo,
    uci_option_config::UCIOptionConfig,
    uci_position::UCIPosition,
    uci_search_control::UCISearchControl,
    uci_time_control::UCITimeControl,
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
}

impl UCIMessage {
    fn parse_debug(tokens: TokenSlice) -> Result<(TokenSlice, Self), ParseUCIMessageError> {
        let debug = tokens.get(0).map(|(s, _)| *s == "on").unwrap_or(true);
        Ok((&[], Self::Debug(debug)))
    }

    fn parse_set_option(
        mut tokens: TokenSlice,
    ) -> Result<(TokenSlice, Self), ParseUCIMessageError> {
        let is_keyword = |t: &str| matches!(t, "name" | "value");

        let mut name = None;
        let mut value = None;

        while !tokens.is_empty() {
            let head = tokens[0].0;
            tokens = &tokens[1..];
            match head {
                "name" => {
                    let (rest, name_str) = parse_string_option(is_keyword, tokens);
                    name = Some(name_str);
                    tokens = rest;
                }
                "value" => {
                    let (rest, value_str) = parse_string_option(is_keyword, tokens);
                    value = Some(value_str);
                    tokens = rest;
                }

                _ => (),
            }
        }

        if let Some(name) = name {
            Ok((&[], Self::SetOption { name, value }))
        } else {
            Err(ParseUCIMessageError)
        }
    }

    fn parse_register(mut tokens: TokenSlice) -> Result<(TokenSlice, Self), ParseUCIMessageError> {
        let is_keyword = |t: &str| matches!(t, "later" | "name" | "code");

        let mut later = false;
        let mut name = None;
        let mut code = None;

        while !tokens.is_empty() {
            let head = tokens[0].0;
            tokens = &tokens[1..];
            match head {
                "later" => later = true,
                "name" => {
                    let (rest, name_str) = parse_string_option(is_keyword, tokens);
                    name = Some(name_str);
                    tokens = rest;
                }
                "code" => {
                    let (rest, code_str) = parse_string_option(is_keyword, tokens);
                    code = Some(code_str);
                    tokens = rest;
                }

                _ => (),
            }
        }

        Ok((&[], Self::Register { later, name, code }))
    }

    fn parse_go(mut tokens: TokenSlice) -> Result<(TokenSlice, Self), ParseUCIMessageError> {
        let mut search_control: Option<UCISearchControl> = None;

        let mut infinite = false;
        let mut move_time = None;

        let mut white_time = None;
        let mut black_time = None;
        let mut white_increment = None;
        let mut black_increment = None;
        let mut moves_to_go = None;

        let mut ponder = false;

        while !tokens.is_empty() {
            let head = tokens[0].0;
            tokens = &tokens[1..];

            match head {
                "searchmoves" => {
                    let search_moves = try_parse_many(&mut tokens);
                    if !search_moves.is_empty() {
                        search_control
                            .get_or_insert_with(Default::default)
                            .search_moves = search_moves;
                    }
                }
                "ponder" => ponder = true,
                "wtime" => {
                    if let Some(ms) = try_parse_next(&mut tokens) {
                        white_time = Some(Duration::from_millis(ms));
                    }
                }
                "btime" => {
                    if let Some(ms) = try_parse_next(&mut tokens) {
                        black_time = Some(Duration::from_millis(ms));
                    }
                }
                "winc" => {
                    if let Some(ms) = try_parse_next(&mut tokens) {
                        white_increment = Some(Duration::from_millis(ms));
                    }
                }
                "binc" => {
                    if let Some(ms) = try_parse_next(&mut tokens) {
                        black_increment = Some(Duration::from_millis(ms));
                    }
                }
                "movestogo" => moves_to_go = try_parse_next(&mut tokens).or(moves_to_go),
                "depth" => {
                    if let Some(depth) = try_parse_next(&mut tokens) {
                        search_control.get_or_insert_with(Default::default).depth = Some(depth);
                    }
                }
                "nodes" => {
                    if let Some(nodes) = try_parse_next(&mut tokens) {
                        search_control.get_or_insert_with(Default::default).nodes = Some(nodes);
                    }
                }
                "mate" => {
                    if let Some(mates) = try_parse_next(&mut tokens) {
                        search_control.get_or_insert_with(Default::default).mate = Some(mates);
                    }
                }
                "movetime" => {
                    if let Some(ms) = try_parse_next(&mut tokens) {
                        move_time = Some(Duration::from_millis(ms));
                    }
                }
                "infinite" => infinite = true,

                _ => (),
            }
        }

        let time_control = if infinite {
            Some(UCITimeControl::Infinite)
        } else if let Some(move_time) = move_time {
            Some(UCITimeControl::MoveTime(move_time))
        } else if white_time.is_some()
            || black_time.is_some()
            || white_increment.is_some()
            || black_increment.is_some()
            || moves_to_go.is_some()
        {
            Some(UCITimeControl::TimeLeft {
                white_time,
                black_time,
                white_increment,
                black_increment,
                moves_to_go,
            })
        } else if ponder {
            Some(UCITimeControl::Ponder)
        } else {
            None
        };

        Ok((
            tokens,
            Self::Go {
                time_control,
                search_control,
            },
        ))
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
                    write!(f, " value {value}")?;
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
        }
    }
}

impl FromStr for UCIMessage {
    type Err = ParseUCIMessageError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tokens = split_tokens(s);
        let mut tokens = &tokens[..];

        while !tokens.is_empty() {
            let head = tokens[0].0;
            tokens = &tokens[1..];

            match head {
                "uci" => return Ok(Self::UCI),

                "debug" => return Ok(Self::parse_debug(tokens)?.1),

                "isready" => return Ok(Self::IsReady),

                "setoption" => return Ok(Self::parse_set_option(tokens)?.1),

                "register" => return Ok(Self::parse_register(tokens)?.1),

                "ucinewgame" => return Ok(Self::UCINewGame),

                "position" => {
                    return Ok(Self::Position(
                        UCIPosition::parse(tokens)
                            .map_err(|_| ParseUCIMessageError)?
                            .1,
                    ))
                }

                "go" => return Ok(Self::parse_go(tokens)?.1),

                "stop" => return Ok(Self::Stop),

                "ponderhit" => return Ok(Self::PonderHit),

                "quit" => return Ok(Self::Quit),

                // Skip this token and look to see if the remainder can be
                // interpreted as a command. This is mandated by the spec
                _ => (),
            }
        }

        Err(ParseUCIMessageError)
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
