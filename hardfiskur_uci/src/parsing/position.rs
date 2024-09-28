use nom::{
    branch::alt,
    combinator::{opt, value},
    multi::{count, many0},
    sequence::preceded,
    IResult,
};

use crate::{UCIPosition, UCIPositionBase};

use super::utils::{token, token_tag, token_uci_move};

pub fn position_body(input: &str) -> IResult<&str, UCIPosition> {
    let (input, base) = position_base(input)?;
    let (input, moves) = opt(preceded(token_tag("moves"), many0(token_uci_move)))(input)?;

    Ok((
        input,
        UCIPosition {
            base,
            moves: moves.unwrap_or_default(),
        },
    ))
}

fn position_base(input: &str) -> IResult<&str, UCIPositionBase> {
    alt((
        value(UCIPositionBase::StartPos, token_tag("startpos")),
        position_base_fen,
    ))(input)
}

fn position_base_fen(input: &str) -> IResult<&str, UCIPositionBase> {
    let (input, _) = token_tag("fen")(input)?;
    let (input, fen_parts) = count(token, 6)(input)?;

    Ok((input, UCIPositionBase::Fen(fen_parts.join(" "))))
}
