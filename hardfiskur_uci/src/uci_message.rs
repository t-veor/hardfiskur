use std::{fmt::Display, str::FromStr, time::Duration};

use hardfiskur_core::board::UCIMove;
use nom::{
    branch::alt,
    combinator::{opt, rest, success, value},
    multi::{many0, many_till},
    sequence::{pair, preceded, tuple},
    IResult, Parser,
};
use nom_permutation::permutation_opt;
use thiserror::Error;

use crate::{
    parse_utils::{
        parser_uci_move, take_tokens_till, token, token_millis, token_tag, token_u32, token_u64,
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
        let (input, value) = opt(preceded(token_tag("value"), rest))(input)?;

        Ok((
            input,
            Self::SetOption {
                name: name.to_string(),
                value: value.map(|s| s.trim().to_string()),
            },
        ))
    }

    fn parser_register_body(input: &str) -> IResult<&str, Self> {
        alt((
            pair(token_tag("register"), token_tag("later")).map(|_| Self::Register {
                later: true,
                name: None,
                code: None,
            }),
            tuple((
                opt(preceded(
                    token_tag("name"),
                    take_tokens_till(token_tag("code")),
                )),
                opt(preceded(token_tag("code"), rest.map(str::trim))),
            ))
            .map(|(name, code)| Self::Register {
                later: false,
                name: name.map(|s| s.to_string()),
                code: code.map(|s| s.to_string()),
            }),
        ))(input)
    }

    fn parser_go_body(input: &str) -> IResult<&str, Self> {
        permutation_opt((
            preceded(token_tag("searchmoves"), many0(parser_uci_move)),
            token_tag("ponder"),
            preceded(token_tag("wtime"), token_millis),
            preceded(token_tag("btime"), token_millis),
            preceded(token_tag("winc"), token_millis),
            preceded(token_tag("binc"), token_millis),
            preceded(token_tag("movestogo"), token_u32),
            preceded(token_tag("depth"), token_u32),
            preceded(token_tag("nodes"), token_u64),
            preceded(token_tag("mate"), token_u32),
            preceded(token_tag("movetime"), token_millis),
            token_tag("infinite"),
        ))
        .map(
            |(
                search_moves,
                ponder,
                white_time,
                black_time,
                white_increment,
                black_increment,
                moves_to_go,
                depth,
                nodes,
                mate,
                move_time,
                infinite,
            )| {
                Self::Go {
                    time_control: UCITimeControl::from_raw(
                        ponder.is_some(),
                        white_time,
                        black_time,
                        white_increment,
                        black_increment,
                        moves_to_go,
                        move_time,
                        infinite.is_some(),
                    ),
                    search_control: UCISearchControl::from_raw(
                        search_moves.unwrap_or_default(),
                        mate,
                        depth,
                        nodes,
                    ),
                }
            },
        )
        .parse(input)
    }

    fn parser_id_body(input: &str) -> IResult<&str, Self> {
        alt((
            preceded(token_tag("name"), rest.map(str::trim)).map(|name| Self::Id {
                name: Some(name.to_string()),
                author: None,
            }),
            preceded(token_tag("author"), rest.map(str::trim)).map(|author| Self::Id {
                name: None,
                author: Some(author.to_string()),
            }),
        ))(input)
    }

    fn parser_best_move_body(input: &str) -> IResult<&str, Self> {
        tuple((
            parser_uci_move,
            opt(preceded(token_tag("ponder"), parser_uci_move)),
        ))
        .map(|(best_move, ponder)| Self::BestMove { best_move, ponder })
        .parse(input)
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
        let (_, (_, message)) =
            dbg!(many_till(token, command_parser).parse(s)).map_err(|_| ParseUCIMessageError)?;

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
