use std::{fmt::Display, str::FromStr, time::Duration};

use hardfiskur_core::board::UCIMove;
use nom::{
    branch::alt,
    character::complete::{u32, u64},
    combinator::{success, value, verify},
    multi::{many0, many_till},
    sequence::preceded,
    IResult, Parser,
};
use thiserror::Error;

use crate::{
    parse_utils::{
        keyworded_options, parser_uci_move, take_tokens_till, token, token_tag, try_opt_once,
    },
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
    fn parser_debug_body(input: &str) -> IResult<&str, Self> {
        let on = match token(input) {
            Ok((_, s)) => s == "on",
            Err(_) => true,
        };

        Ok(("", Self::Debug(on)))
    }

    fn parser_set_option_body(input: &str) -> IResult<&str, Self> {
        let (input, _) = token_tag("name")(input)?;
        let (input, name) = take_tokens_till(token_tag("value"))(input)?;

        let (input, value) = if let Ok((input, _)) = token_tag("value")(input) {
            (input, Some(input.to_string()))
        } else {
            (input, None)
        };

        Ok((
            input,
            Self::SetOption {
                name: name.to_string(),
                value,
            },
        ))
    }

    fn parser_register_body(input: &str) -> IResult<&str, Self> {
        if token_tag("later")(input).is_ok() {
            // Shortcut, return later = true
            return Ok((
                "",
                Self::Register {
                    later: true,
                    name: None,
                    code: None,
                },
            ));
        }

        let mut name = None;
        let mut code = None;

        let mut name_found = false;
        let mut code_found = false;

        let (mut input, _) =
            take_tokens_till(verify(token, |s| matches!(s, "name" | "code")))(input)?;

        while !input.is_empty() {
            match verify(token, |s| matches!(s, "name" | "code"))(input) {
                Ok((rest, t)) => {
                    match t {
                        "name" => name_found = true,
                        _ => code_found = true,
                    }

                    let (rest, value) = take_tokens_till(verify(token, |s: &str| {
                        !name_found && s == "name" || !code_found && s == "code"
                    }))(rest)?;

                    match t {
                        "name" => name = Some(value.to_string()),
                        _ => code = Some(value.to_string()),
                    }

                    input = rest;
                }

                // Must be EOF here, as we always go until we hit "name", "code", or EOF
                Err(_) => break,
            }
        }

        Ok((
            input,
            Self::Register {
                later: false,
                name,
                code,
            },
        ))
    }

    fn parser_go_body(input: &str) -> IResult<&str, Self> {
        let is_keyword = verify(token, |&s| {
            matches!(
                s,
                "searchmoves"
                    | "ponder"
                    | "wtime"
                    | "btime"
                    | "winc"
                    | "binc"
                    | "movestogo"
                    | "depth"
                    | "nodes"
                    | "mate"
                    | "movetime"
                    | "infinite"
            )
        });

        let (input, options) = keyworded_options(is_keyword)(input)?;

        let mut search_moves = Vec::new();
        let mut mate = None;
        let mut depth = None;
        let mut nodes = None;

        let mut infinite = false;
        let mut move_time = None;

        let mut white_time = None;
        let mut black_time = None;
        let mut white_increment = None;
        let mut black_increment = None;
        let mut moves_to_go = None;

        let mut ponder = false;

        for (option_name, value) in options {
            match option_name {
                "searchmoves" => {
                    search_moves = try_opt_once(many0(parser_uci_move), value).unwrap_or_default()
                }
                "ponder" => ponder = true,
                "wtime" => white_time = try_opt_once(u64.map(Duration::from_millis), value),
                "btime" => black_time = try_opt_once(u64.map(Duration::from_millis), value),
                "winc" => white_increment = try_opt_once(u64.map(Duration::from_millis), value),
                "binc" => black_increment = try_opt_once(u64.map(Duration::from_millis), value),
                "movestogo" => moves_to_go = try_opt_once(u32, value),
                "depth" => depth = try_opt_once(u32, value),
                "nodes" => nodes = try_opt_once(u64, value),
                "mate" => mate = try_opt_once(u32, value),
                "movetime" => move_time = try_opt_once(u64.map(Duration::from_millis), value),
                "infinite" => infinite = true,
                _ => unreachable!(),
            }
        }

        let search_control =
            if !search_moves.is_empty() || mate.is_some() || depth.is_some() || nodes.is_some() {
                Some(UCISearchControl {
                    search_moves,
                    mate,
                    depth,
                    nodes,
                })
            } else {
                None
            };

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
            input,
            Self::Go {
                time_control,
                search_control,
            },
        ))
    }

    fn parser_id_body(input: &str) -> IResult<&str, Self> {
        let (input, id_type) = alt((token_tag("name"), token_tag("author")))(input)?;

        let value = input.trim_ascii().to_string();

        Ok((
            "",
            match id_type {
                "name" => Self::Id {
                    name: Some(value),
                    author: None,
                },
                "author" => Self::Id {
                    name: None,
                    author: Some(value),
                },
                _ => unreachable!(),
            },
        ))
    }

    fn parser_best_move_body(input: &str) -> IResult<&str, Self> {
        let (input, best_move) = parser_uci_move(input)?;

        let (input, ponder) = match token_tag("ponder")(input) {
            Ok((input, _)) => {
                let (input, ponder_move) = parser_uci_move(input)?;
                (input, Some(ponder_move))
            }
            Err(_) => (input, None),
        };

        Ok((input, Self::BestMove { best_move, ponder }))
    }
}

impl FromStr for UCIMessage {
    type Err = ParseUCIMessageError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let command_parser = alt((
            // gui -> engine commands
            preceded(token_tag("uci"), success(Self::UCI)),
            preceded(token_tag("debug"), Self::parser_debug_body),
            preceded(token_tag("isready"), success(Self::IsReady)),
            preceded(token_tag("setoption"), Self::parser_set_option_body),
            preceded(token_tag("register"), Self::parser_register_body),
            preceded(token_tag("ucinewgame"), success(Self::UCINewGame)),
            preceded(
                token_tag("position"),
                UCIPosition::parser.map(Self::Position),
            ),
            preceded(token_tag("go"), Self::parser_go_body),
            preceded(token_tag("stop"), success(Self::Stop)),
            preceded(token_tag("ponderhit"), success(Self::PonderHit)),
            preceded(token_tag("quit"), success(Self::Quit)),
            // engine -> gui commands
            preceded(token_tag("id"), Self::parser_id_body),
            preceded(token_tag("uciok"), success(Self::UCIOk)),
            preceded(token_tag("readyok"), success(Self::ReadyOk)),
            preceded(token_tag("bestmove"), Self::parser_best_move_body),
            preceded(
                token_tag("copyprotection"),
                ProtectionState::parser.map(Self::CopyProtection),
            ),
            preceded(
                token_tag("registration"),
                ProtectionState::parser.map(Self::Registration),
            ),
            preceded(token_tag("info"), UCIInfo::parser.map(Self::Info)),
            preceded(
                token_tag("option"),
                UCIOptionConfig::parser.map(Self::Option),
            ),
        ));

        // many_till(token, ...) skips any initial tokens it couldn't parse.
        // This behavior is mandated by the spec
        let (_, (_, message)) = many_till(token, command_parser)
            .parse(s)
            .map_err(|_| ParseUCIMessageError)?;

        // Ignore unparseable stuff afterwards
        Ok(message)
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtectionState {
    Checking,
    Ok,
    Error,
}

impl ProtectionState {
    fn parser(input: &str) -> IResult<&str, Self> {
        alt((
            value(Self::Checking, token_tag("checking")),
            value(Self::Ok, token_tag("ok")),
            value(Self::Error, token_tag("error")),
        ))(input)
    }
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
