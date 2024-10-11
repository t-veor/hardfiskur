use std::{fmt::Display, time::Duration};

use hardfiskur_core::board::UCIMove;
use hardfiskur_engine::{score::Score, search_result::SearchInfo};

use crate::format_utils::SpaceSepFormatter;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UCIInfoScore {
    pub cp: Option<i32>,
    pub mate: Option<i32>,
    pub lower_bound: bool,
    pub upper_bound: bool,
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

impl From<Score> for UCIInfoScore {
    fn from(score: Score) -> Self {
        Self {
            cp: score.as_centipawns(),
            mate: score.as_mate_in(),
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UCIInfoCurrLine {
    pub cpu_nr: Option<u32>,
    pub moves: Vec<UCIMove>,
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

impl From<SearchInfo> for UCIInfo {
    fn from(value: SearchInfo) -> Self {
        Self {
            score: Some(value.score.into()),
            depth: Some(value.raw_stats.depth.into()),
            sel_depth: Some(value.raw_stats.sel_depth.into()),
            nodes: Some(value.raw_stats.nodes_searched),
            tb_hits: Some(value.raw_stats.tt_hits),
            time: Some(value.elapsed),
            pv: value.pv.iter().map(|m| UCIMove::from(*m)).collect(),
            hash_full: Some(value.hash_full.try_into().unwrap_or(1000)),
            ..Default::default()
        }
    }
}
