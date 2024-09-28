use std::{str::FromStr, time::Duration};

use hardfiskur_core::board::UCIMove;
use nom::{
    bytes::complete::take_till1,
    character::complete::{space0, u64},
    combinator::fail,
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
    let (input, following_ws) = space0(input)?;

    let total_len = preceding_ws.len() + result.len() + following_ws.len();

    Ok((input, (result, total_len)))
}

pub fn token_tag<'a>(tag: &'a str) -> impl Fn(&str) -> IResult<&str, &str> + 'a {
    move |input: &str| -> IResult<&str, &str> {
        let (rest, t) = token(input)?;
        if t == tag {
            Ok((rest, t))
        } else {
            Err(nom::Err::Error(error_position!(input, ErrorKind::Tag)))
        }
    }
}

pub fn keyworded_options<'a, E: ParseError<&'a str>>(
    mut keyword_parser: impl Parser<&'a str, &'a str, E>,
) -> impl FnMut(&'a str) -> IResult<&'a str, Vec<(&'a str, &'a str)>> {
    move |mut input: &str| {
        let mut options = Vec::new();

        while !input.is_empty() {
            let rest = if let Ok((rest, keyword)) = keyword_parser.parse(input) {
                let (rest, value) = take_tokens_till_ref(&mut keyword_parser)(rest)?;
                options.push((keyword, value));
                rest
            } else {
                // This should only happen on the first iteration -- just skip
                // tokens until we get to a keyword.
                let (rest, _) = take_tokens_till_ref(&mut keyword_parser)(input)?;
                rest
            };
            input = rest;
        }

        Ok((input, options))
    }
}

pub fn take_tokens_till<'a, E: ParseError<&'a str>>(
    mut recognizer: impl Parser<&'a str, &'a str, E>,
) -> impl FnMut(&'a str) -> IResult<&'a str, &'a str> {
    move |input: &str| take_tokens_till_ref(&mut recognizer)(input)
}

pub fn take_tokens_till_ref<'a, E: ParseError<&'a str>>(
    recognizer: &mut impl Parser<&'a str, &'a str, E>,
) -> impl FnMut(&'a str) -> IResult<&'a str, &'a str> + '_ {
    move |original_input: &str| {
        let mut input = original_input;

        let mut curr_token_length = 0;

        while !input.is_empty() {
            if let Ok(_) = recognizer.parse(input) {
                break;
            } else {
                let (rest, (_, token_len)) = token_and_len(input)?;
                curr_token_length += token_len;
                input = rest;
            }
        }

        Ok((input, original_input[..curr_token_length].trim()))
    }
}

pub fn parser_uci_move(input: &str) -> IResult<&str, UCIMove> {
    let (input, t) = token(input)?;
    match UCIMove::from_str(t) {
        Ok(m) => Ok((input, m)),
        Err(_) => context("expected valid uci move", fail)(input),
    }
}

pub fn try_opt_once<'a, O>(
    mut parser: impl Parser<&'a str, O, Error<&'a str>>,
    input: &'a str,
) -> Option<O> {
    match parser.parse(input) {
        Ok((_, value)) => Some(value),
        Err(_) => None,
    }
}

pub fn millis(input: &str) -> IResult<&str, Duration> {
    // Split up to help the type inference
    let (input, x) = u64(input)?;
    Ok((input, Duration::from_millis(x)))
}
