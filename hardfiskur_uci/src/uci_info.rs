use std::{fmt::Display, time::Duration};

use hardfiskur_core::board::UCIMove;
use nom::{
    branch::alt,
    character::complete::{i32, u32, u64},
    combinator::{opt, recognize, rest, verify},
    multi::many0,
    sequence::pair,
    IResult, Parser,
};

use crate::{
    format_utils::SpaceSepFormatter,
    parse_utils::{keyworded_options, parser_uci_move, token, token_tag, try_opt_once},
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
        let is_keyword = verify(token, |s: &str| {
            matches!(s, "cp" | "mate" | "lowerbound" | "upperbound")
        });

        let (input, options) = keyworded_options(is_keyword)(input)?;

        let mut score = Self::default();
        for (option_name, value) in options {
            match option_name {
                "cp" => score.cp = try_opt_once(i32, value),
                "mate" => score.mate = try_opt_once(i32, value),
                "lowerbound" => score.lower_bound = true,
                "upperbound" => score.upper_bound = true,
                _ => unreachable!(),
            }
        }

        Ok((input, score))
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
    pub hashfull: Option<u32>,
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
        let is_keyword = alt((
            verify(token, |s: &str| {
                matches!(
                    s,
                    "depth"
                        | "seldepth"
                        | "time"
                        | "nodes"
                        | "pv"
                        | "multipv"
                        | "score"
                        | "currmove"
                        | "currmovenumber"
                        | "hashfull"
                        | "nps"
                        | "tbhits"
                        | "sbhits"
                        | "cpuload"
                        | "refutation"
                        | "currline"
                )
            }),
            // Special "string" parser, just match the entire rest of the line
            recognize(pair(token_tag("string"), rest)),
        ));

        let (input, options) = keyworded_options(is_keyword)(input)?;

        let mut info = Self::default();

        for (option_name, value) in options {
            match option_name {
                "depth" => info.depth = try_opt_once(u32, value),
                "seldepth" => info.sel_depth = try_opt_once(u32, value),
                "time" => info.time = try_opt_once(u64.map(Duration::from_millis), value),
                "nodes" => info.nodes = try_opt_once(u64, value),
                "pv" => info.pv = try_opt_once(many0(parser_uci_move), value).unwrap_or_default(),
                "multipv" => info.multi_pv = try_opt_once(u32, value),
                "score" => info.score = try_opt_once(UCIInfoScore::parser, value),
                "currmove" => info.curr_move = try_opt_once(parser_uci_move, value),
                "currmovenumber" => info.curr_move_number = try_opt_once(u32, value),
                "hashfull" => info.hashfull = try_opt_once(u32, value),
                "nps" => info.nps = try_opt_once(u64, value),
                "tbhits" => info.tb_hits = try_opt_once(u64, value),
                "sbhits" => info.sb_hits = try_opt_once(u64, value),
                "cpuload" => info.cpu_load = try_opt_once(u32, value),
                "refutation" => {
                    info.refutation =
                        try_opt_once(many0(parser_uci_move), value).unwrap_or_default()
                }
                "currline" => info.curr_line = try_opt_once(UCIInfoCurrLine::parser, value),

                // special string tag
                _ => {
                    info.string = Some(
                        option_name
                            .strip_prefix("string")
                            .unwrap_or(option_name)
                            .trim_ascii()
                            .to_string(),
                    );
                }
            }
        }

        Ok((input, info))
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
        formatter.push_option("hashfull", self.hashfull)?;
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
