use std::str::FromStr;

pub type Token<'a> = (&'a str, &'a str);
pub type TokenSlice<'a> = &'a [Token<'a>];

pub fn split_tokens(s: &str) -> Vec<Token> {
    let s = s.trim_ascii_start();

    let mut spans = Vec::new();

    let mut last_span_start = 0;
    let mut last_is_whitespace = false;

    for (i, b) in s.bytes().enumerate() {
        if last_is_whitespace != b.is_ascii_whitespace() {
            if last_span_start < i {
                let span = &s[last_span_start..i];
                spans.push(span);
            }

            last_is_whitespace = b.is_ascii_whitespace();
            last_span_start = i;
        }
    }

    if last_span_start < s.len() {
        let span = &s[last_span_start..];
        spans.push(span);
    }

    spans
        .chunks(2)
        .map(|pair| match pair {
            &[word, whitespace] => (word, whitespace),
            &[word] => (word, ""),
            _ => unreachable!(),
        })
        .collect()
}

pub fn join_tokens(tokens: &[Token]) -> String {
    tokens.iter().copied().flat_map(|(a, b)| [a, b]).collect()
}

pub fn try_parse_next<T: FromStr>(tokens: &mut TokenSlice) -> Option<T> {
    if let Some(value) = tokens.get(0).and_then(|(s, _)| s.parse().ok()) {
        *tokens = &tokens[1..];
        Some(value)
    } else {
        None
    }
}

pub fn try_parse_many<T: FromStr>(tokens: &mut TokenSlice) -> Vec<T> {
    let mut values = Vec::new();

    while let Some(value) = try_parse_next(tokens) {
        values.push(value);
    }

    values
}

pub fn take_until(
    predicate: impl Fn(Token) -> bool,
    tokens: TokenSlice,
) -> (TokenSlice, TokenSlice) {
    for (i, token) in tokens.iter().enumerate() {
        if predicate(*token) {
            return (&tokens[i..], &tokens[..i]);
        }
    }

    (&[], tokens)
}

pub fn parse_string_option(
    keyword_predicate: impl Fn(&str) -> bool,
    tokens: TokenSlice,
) -> (TokenSlice, String) {
    let (rest, string_tokens) = take_until(|t| keyword_predicate(t.0), tokens);

    (
        rest,
        join_tokens(string_tokens).trim_ascii_end().to_string(),
    )
}
