use std::str::FromStr;

use hardfiskur_core::board::UCIMove;
use nom::{
    bytes::complete::take_till1,
    character::complete::space0,
    combinator::fail,
    error::{context, Error, ErrorKind, ParseError},
    error_position, IResult, Parser,
};

pub fn token(input: &str) -> IResult<&str, &str> {
    let (input, (_, result, _)) = token_full(input)?;
    Ok((input, result))
}

pub fn token_full(input: &str) -> IResult<&str, (&str, &str, &str)> {
    let (input, preceding_ws) = space0(input)?;
    let (input, result) = take_till1(|c: char| c.is_ascii_whitespace())(input)?;
    let (input, following_ws) = space0(input)?;

    Ok((input, (preceding_ws, result, following_ws)))
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
                let (rest, (a, b, c)) = token_full(input)?;
                curr_token_length += a.len() + b.len() + c.len();
                input = rest;
            }
        }

        Ok((input, original_input[..curr_token_length].trim_ascii()))
    }
}

pub fn parser_uci_move(input: &str) -> IResult<&str, UCIMove> {
    let (input, t) = token(input)?;
    match UCIMove::from_str(t) {
        Ok(m) => Ok((input, m)),
        Err(_) => context("expected valid uci move", fail)(input),
    }
}

pub fn try_parse_and_discard_rest<'a, O>(
    mut parser: impl Parser<&'a str, O, Error<&'a str>>,
    input: &'a str,
) -> Option<O> {
    match parser.parse(input) {
        Ok((_, value)) => Some(value),
        Err(_) => None,
    }
}
