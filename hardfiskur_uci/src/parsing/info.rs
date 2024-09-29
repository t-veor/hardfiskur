use nom::{
    branch::alt,
    combinator::{opt, rest, success},
    multi::many0,
    sequence::{pair, preceded},
    IResult, Parser,
};
use nom_permutation::permutation_opt;

use super::utils::{
    token_i32, token_millis_ignore_negative, token_tag, token_u32, token_u64,
    token_uci_move,
};
use crate::{UCIInfo, UCIInfoCurrLine, UCIInfoScore};

fn info_score(input: &str) -> IResult<&str, UCIInfoScore> {
    pair(
        alt((
            preceded(token_tag("cp"), token_i32).map(|cp| (Some(cp), None)),
            preceded(token_tag("mate"), token_i32).map(|mate| (None, Some(mate))),
        )),
        alt((
            token_tag("lowerbound").map(|_| (true, false)),
            token_tag("upperbound").map(|_| (false, true)),
            success((false, false)),
        )),
    )
    .map(|((cp, mate), (lower_bound, upper_bound))| UCIInfoScore {
        cp,
        mate,
        lower_bound,
        upper_bound,
    })
    .parse(input)
}

fn info_curr_line(input: &str) -> IResult<&str, UCIInfoCurrLine> {
    let (input, cpu_nr) = opt(token_u32)(input)?;
    let (input, moves) = many0(token_uci_move)(input)?;

    Ok((input, UCIInfoCurrLine { cpu_nr, moves }))
}

pub fn info_body(input: &str) -> IResult<&str, UCIInfo> {
    permutation_opt((
        preceded(token_tag("depth"), token_u32),
        preceded(token_tag("seldepth"), token_u32),
        preceded(token_tag("time"), token_millis_ignore_negative),
        preceded(token_tag("nodes"), token_u64),
        preceded(token_tag("pv"), many0(token_uci_move)),
        preceded(token_tag("multipv"), token_u32),
        preceded(token_tag("score"), info_score),
        preceded(token_tag("currmove"), token_uci_move),
        preceded(token_tag("currmovenumber"), token_u32),
        preceded(token_tag("hashfull"), token_u32),
        preceded(token_tag("nps"), token_u64),
        preceded(token_tag("tbhits"), token_u64),
        preceded(token_tag("sbhits"), token_u64),
        preceded(token_tag("cpuload"), token_u32),
        preceded(token_tag("refutation"), many0(token_uci_move)),
        preceded(token_tag("currline"), info_curr_line),
        preceded(token_tag("string"), rest.map(str::trim)),
    ))
    .map(
        |(
            depth,
            sel_depth,
            time,
            nodes,
            pv,
            multi_pv,
            score,
            curr_move,
            curr_move_number,
            hash_full,
            nps,
            tb_hits,
            sb_hits,
            cpu_load,
            refutation,
            curr_line,
            string,
        )| UCIInfo {
            depth,
            sel_depth,
            time: time.flatten(),
            nodes,
            pv: pv.unwrap_or_default(),
            multi_pv,
            score,
            curr_move,
            curr_move_number,
            hash_full,
            nps,
            tb_hits,
            sb_hits,
            cpu_load,
            refutation: refutation.unwrap_or_default(),
            curr_line,
            string: string.map(|s| s.to_string()),
        },
    )
    .parse(input)
}
