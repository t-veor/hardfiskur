use nom::{
    branch::alt,
    combinator::{map_opt, opt, rest},
    multi::many0,
    sequence::{preceded, tuple},
    IResult, Parser,
};

use crate::UCIOptionConfig;

use super::utils::{take_tokens_till, take_tokens_until, token_i64, token_tag};

pub fn option_body(input: &str) -> IResult<&str, UCIOptionConfig> {
    let parser = tuple((
        preceded(token_tag("name"), take_tokens_till(token_tag("type"))),
        preceded(
            token_tag("type"),
            alt((
                token_tag("check"),
                token_tag("spin"),
                token_tag("combo"),
                token_tag("string"),
                token_tag("button"),
            )),
        ),
        opt(preceded(
            token_tag("default"),
            alt((
                take_tokens_until(token_tag("min")),
                take_tokens_until(token_tag("max")),
                take_tokens_until(token_tag("var")),
                rest.map(str::trim),
            )),
        )),
        opt(preceded(token_tag("min"), token_i64)),
        opt(preceded(token_tag("max"), token_i64)),
        many0(preceded(
            token_tag("var"),
            take_tokens_till(token_tag("var")),
        )),
    ));

    map_opt(parser, |(name, option_type, default, min, max, var)| {
        UCIOptionConfig::from_raw(name, option_type, default, min, max, var)
    })(input)
}
