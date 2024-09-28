use std::{fmt::Display, time::Duration};

use hardfiskur_core::board::UCIMove;
use nom::{
    branch::alt,
    character::complete::{i32, u32, u64},
    combinator::{opt, recognize, rest, success, verify},
    multi::many0,
    sequence::{pair, preceded},
    IResult, Parser,
};
use nom_permutation::permutation_opt;

use crate::{
    format_utils::SpaceSepFormatter,
    parse_utils::{keyworded_options, millis, parser_uci_move, token, token_tag, try_opt_once},
};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UCIInfoScore {
    pub cp: Option<i32>,
    pub mate: Option<i32>,
    pub lower_bound: bool,
    pub upper_bound: bool,
}

impl UCIInfoScore {
    pub fn parser(input: &str) -> IResult<&str, Self> {
        pair(
            alt((
                preceded(token_tag("cp"), i32).map(|cp| (Some(cp), None)),
                preceded(token_tag("mate"), i32).map(|mate| (None, Some(mate))),
            )),
            alt((
                token_tag("lowerbound").map(|_| (true, false)),
                token_tag("upperbound").map(|_| (false, true)),
                success((false, false)),
            )),
        )
        .map(|((cp, mate), (lower_bound, upper_bound))| Self {
            cp,
            mate,
            lower_bound,
            upper_bound,
        })
        .parse(input)
    }
}

impl Display for UCIInfoScore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut formatter = SpaceSepFormatter::new(f);

        formatter.push_option("cp", self.cp)?;
        formatter.push_option("mate", self.mate)?;

        if self.lower_bound {
            formatter.push_str("lowerbound")?;
        }

        if self.upper_bound {
            formatter.push_str("upperbound")?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UCIInfoCurrLine {
    pub cpu_nr: Option<u32>,
    pub moves: Vec<UCIMove>,
}

impl UCIInfoCurrLine {
    pub fn parser(input: &str) -> IResult<&str, Self> {
        let (input, cpu_nr) = opt(u32)(input)?;
        let (input, moves) = many0(parser_uci_move)(input)?;

        Ok((input, Self { cpu_nr, moves }))
    }
}

impl Display for UCIInfoCurrLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "currline")?;

        if let Some(cpu_nr) = self.cpu_nr {
            write!(f, " {cpu_nr}")?;
        }

        for m in self.moves.iter() {
            write!(f, " {m}")?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UCIInfo {
    pub depth: Option<u32>,
    pub sel_depth: Option<u32>,
    pub time: Option<Duration>,
    pub nodes: Option<u64>,
    pub pv: Vec<UCIMove>,
    pub multi_pv: Option<u32>,
    pub score: Option<UCIInfoScore>,
    pub curr_move: Option<UCIMove>,
    pub curr_move_number: Option<u32>,
    pub hash_full: Option<u32>,
    pub nps: Option<u64>,
    pub tb_hits: Option<u64>,
    pub sb_hits: Option<u64>,
    pub cpu_load: Option<u32>,
    pub string: Option<String>,
    pub refutation: Vec<UCIMove>,
    pub curr_line: Option<UCIInfoCurrLine>,
}

impl UCIInfo {
    pub fn parser(input: &str) -> IResult<&str, Self> {
        permutation_opt((
            preceded(token_tag("depth"), u32),
            preceded(token_tag("seldepth"), u32),
            preceded(token_tag("time"), millis),
            preceded(token_tag("nodes"), u64),
            preceded(token_tag("pv"), many0(parser_uci_move)),
            preceded(token_tag("multipv"), u32),
            preceded(token_tag("score"), UCIInfoScore::parser),
            preceded(token_tag("currmove"), parser_uci_move),
            preceded(token_tag("currmovenumber"), u32),
            preceded(token_tag("hashfull"), u32),
            preceded(token_tag("nps"), u64),
            preceded(token_tag("tbhits"), u64),
            preceded(token_tag("sbhits"), u64),
            preceded(token_tag("cpuload"), u32),
            preceded(token_tag("refutation"), many0(parser_uci_move)),
            preceded(token_tag("currline"), UCIInfoCurrLine::parser),
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
            )| Self {
                depth,
                sel_depth,
                time,
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
}

impl Display for UCIInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut formatter = SpaceSepFormatter::new(f);

        formatter.push_option("depth", self.depth)?;
        formatter.push_option("seldepth", self.sel_depth)?;
        formatter.push_option("time", self.time.map(|d| d.as_millis()))?;
        formatter.push_option("nodes", self.nodes)?;

        if !self.pv.is_empty() {
            formatter.push_str("pv")?;
            for m in self.pv.iter() {
                formatter.push(m)?;
            }
        }

        formatter.push_option("multipv", self.multi_pv)?;
        formatter.push_option("score", self.score.as_ref())?;
        formatter.push_option("currmove", self.curr_move)?;
        formatter.push_option("currmovenumber", self.curr_move_number)?;
        formatter.push_option("hashfull", self.hash_full)?;
        formatter.push_option("nps", self.nps)?;
        formatter.push_option("tbhits", self.tb_hits)?;
        formatter.push_option("sbhits", self.sb_hits)?;
        formatter.push_option("cpuload", self.cpu_load)?;

        if !self.refutation.is_empty() {
            formatter.push_str("refutation")?;
            for m in self.refutation.iter() {
                formatter.push(m)?;
            }
        }

        formatter.push_option("currline", self.curr_line.as_ref())?;

        formatter.push_option("string", self.string.as_ref())?;

        Ok(())
    }
}
