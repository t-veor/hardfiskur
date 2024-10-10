use std::{str::FromStr, time::Duration};

use hardfiskur_core::board::UCIMove;
use nom::{
    branch::alt,
    bytes::complete::take_till1,
    character::complete::{i32, i64, space0, space1, u32, u64},
    combinator::{eof, fail, rest},
    error::{context, Error, ErrorKind, ParseError},
    error_position, IResult, Parser,
};

pub fn token(input: &str) -> IResult<&str, &str> {
    let (input, (result, _)) = token_and_len(input)?;
    Ok((input, result))
}

pub fn token_and_len(input: &str) -> IResult<&str, (&str, usize)> {
    let (input, preceding_ws) = space0(input)?;
    let (input, result) = take_till1(|c: char| c.is_whitespace())(input)?;
    let (input, following_ws) = alt((space1, eof))(input)?;

    let total_len = preceding_ws.len() + result.len() + following_ws.len();

    Ok((input, (result, total_len)))
}

pub fn token_tag(tag: &str) -> impl Fn(&str) -> IResult<&str, &str> + '_ {
    move |input: &str| -> IResult<&str, &str> {
        let (rest, t) = token(input)?;
        if t == tag {
            Ok((rest, t))
        } else {
            Err(nom::Err::Error(error_position!(input, ErrorKind::Tag)))
        }
    }
}

pub fn tokenize<'a, O>(
    mut parser: impl Parser<&'a str, O, Error<&'a str>>,
) -> impl FnMut(&'a str) -> IResult<&'a str, O> {
    move |input: &str| {
        let (input, t) = token(input)?;
        // all_consuming seems to want to move the parser into itself, but this
        // works...
        let (rest, parsed) = parser.parse(t)?;

        if rest.is_empty() {
            Ok((input, parsed))
        } else {
            Err(nom::Err::Error(Error::from_error_kind(
                input,
                ErrorKind::Eof,
            )))
        }
    }
}

pub fn take_tokens_till<'a, E: ParseError<&'a str>>(
    recognizer: impl Parser<&'a str, &'a str, E>,
) -> impl FnMut(&'a str) -> IResult<&'a str, &'a str> {
    alt((take_tokens_until(recognizer), rest))
}

pub fn take_tokens_until<'a, E: ParseError<&'a str>>(
    mut recognizer: impl Parser<&'a str, &'a str, E>,
) -> impl FnMut(&'a str) -> IResult<&'a str, &'a str> {
    move |original_input: &str| {
        let mut input = original_input;

        let mut curr_token_length = 0;

        while !input.is_empty() {
            if recognizer.parse(input).is_ok() {
                return Ok((input, original_input[..curr_token_length].trim()));
            } else {
                let (rest, (_, token_len)) = token_and_len(input)?;
                curr_token_length += token_len;
                input = rest;
            }
        }

        fail(original_input)
    }
}

pub fn token_uci_move(input: &str) -> IResult<&str, UCIMove> {
    let (input, t) = token(input)?;
    match UCIMove::from_str(t) {
        Ok(m) => Ok((input, m)),
        Err(_) => context("expected valid uci move", fail)(input),
    }
}

pub fn token_i32(input: &str) -> IResult<&str, i32> {
    tokenize(i32)(input)
}

pub fn token_i64(input: &str) -> IResult<&str, i64> {
    tokenize(i64)(input)
}

pub fn token_u32(input: &str) -> IResult<&str, u32> {
    tokenize(u32)(input)
}

pub fn token_u64(input: &str) -> IResult<&str, u64> {
    tokenize(u64)(input)
}

pub fn token_millis_ignore_negative(input: &str) -> IResult<&str, Option<Duration>> {
    let (input, millis) = token_i64(input)?;
    if millis < 0 {
        Ok((input, None))
    } else {
        Ok((input, Some(Duration::from_millis(millis as u64))))
    }
}
