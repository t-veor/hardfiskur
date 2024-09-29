mod info;
mod option;
mod position;
pub mod utils;

#[cfg(test)]
mod test;

use info::info_body;
use nom::{
    branch::alt,
    combinator::{opt, rest, success, value},
    multi::{many0, many_till},
    sequence::{preceded, tuple},
    IResult, Parser,
};
use nom_permutation::permutation_opt;
use option::option_body;
use position::position_body;
use utils::{
    take_tokens_till, token, token_millis_ignore_negative, token_tag, token_u32, token_u64,
    token_uci_move,
};

use crate::{uci_message::ProtectionState, UCIMessage, UCISearchControl, UCITimeControl};

fn debug_body(input: &str) -> IResult<&str, UCIMessage> {
    let on = match token(input) {
        Ok((_, s)) => s == "on",
        Err(_) => true,
    };

    Ok(("", UCIMessage::Debug(on)))
}

fn set_option_body(input: &str) -> IResult<&str, UCIMessage> {
    let (input, _) = token_tag("name")(input)?;
    let (input, name) = take_tokens_till(token_tag("value"))(input)?;
    let (input, value) = opt(preceded(token_tag("value"), rest))(input)?;

    Ok((
        input,
        UCIMessage::SetOption {
            name: name.to_string(),
            value: value.map(|s| s.trim().to_string()),
        },
    ))
}

fn register_body(input: &str) -> IResult<&str, UCIMessage> {
    alt((
        token_tag("later").map(|_| UCIMessage::Register {
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
        .map(|(name, code)| UCIMessage::Register {
            later: false,
            name: name.map(|s| s.to_string()),
            code: code.map(|s| s.to_string()),
        }),
    ))(input)
}

fn go_body(input: &str) -> IResult<&str, UCIMessage> {
    permutation_opt((
        preceded(token_tag("searchmoves"), many0(token_uci_move)),
        token_tag("ponder"),
        preceded(token_tag("wtime"), token_millis_ignore_negative),
        preceded(token_tag("btime"), token_millis_ignore_negative),
        preceded(token_tag("winc"), token_millis_ignore_negative),
        preceded(token_tag("binc"), token_millis_ignore_negative),
        preceded(token_tag("movestogo"), token_u32),
        preceded(token_tag("depth"), token_u32),
        preceded(token_tag("nodes"), token_u64),
        preceded(token_tag("mate"), token_u32),
        preceded(token_tag("movetime"), token_millis_ignore_negative),
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
            UCIMessage::Go {
                time_control: UCITimeControl::from_raw(
                    ponder.is_some(),
                    white_time.flatten(),
                    black_time.flatten(),
                    white_increment.flatten(),
                    black_increment.flatten(),
                    moves_to_go,
                    move_time.flatten(),
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
fn id_body(input: &str) -> IResult<&str, UCIMessage> {
    alt((
        preceded(token_tag("name"), rest.map(str::trim)).map(|name| UCIMessage::Id {
            name: Some(name.to_string()),
            author: None,
        }),
        preceded(token_tag("author"), rest.map(str::trim)).map(|author| UCIMessage::Id {
            name: None,
            author: Some(author.to_string()),
        }),
    ))(input)
}

fn best_move_body(input: &str) -> IResult<&str, UCIMessage> {
    tuple((
        token_uci_move,
        opt(preceded(token_tag("ponder"), token_uci_move)),
    ))
    .map(|(best_move, ponder)| UCIMessage::BestMove { best_move, ponder })
    .parse(input)
}

fn protection_state(input: &str) -> IResult<&str, ProtectionState> {
    alt((
        value(ProtectionState::Checking, token_tag("checking")),
        value(ProtectionState::Ok, token_tag("ok")),
        value(ProtectionState::Error, token_tag("error")),
    ))(input)
}

pub fn uci_message(input: &str) -> IResult<&str, UCIMessage> {
    let command_parser = alt((
        // gui -> engine commands
        preceded(token_tag("uci"), success(UCIMessage::UCI)),
        preceded(token_tag("debug"), debug_body),
        preceded(token_tag("isready"), success(UCIMessage::IsReady)),
        preceded(token_tag("setoption"), set_option_body),
        preceded(token_tag("register"), register_body),
        preceded(token_tag("ucinewgame"), success(UCIMessage::UCINewGame)),
        preceded(
            token_tag("position"),
            position_body.map(UCIMessage::Position),
        ),
        preceded(token_tag("go"), go_body),
        preceded(token_tag("stop"), success(UCIMessage::Stop)),
        preceded(token_tag("ponderhit"), success(UCIMessage::PonderHit)),
        preceded(token_tag("quit"), success(UCIMessage::Quit)),
        // engine -> gui commands
        preceded(token_tag("id"), id_body),
        preceded(token_tag("uciok"), success(UCIMessage::UCIOk)),
        preceded(token_tag("readyok"), success(UCIMessage::ReadyOk)),
        preceded(token_tag("bestmove"), best_move_body),
        preceded(
            token_tag("copyprotection"),
            protection_state.map(UCIMessage::CopyProtection),
        ),
        preceded(
            token_tag("registration"),
            protection_state.map(UCIMessage::Registration),
        ),
        preceded(token_tag("info"), info_body.map(UCIMessage::Info)),
        preceded(token_tag("option"), option_body.map(UCIMessage::Option)),
    ));

    // many_till(token, ...) skips any initial tokens it couldn't parse.
    // This behavior is mandated by the spec.
    let (input, (_, message)) = many_till(token, command_parser).parse(input)?;

    Ok((input, message))
}
