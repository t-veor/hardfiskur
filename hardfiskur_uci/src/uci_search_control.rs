use std::fmt::Display;

use hardfiskur_core::board::UCIMove;

use crate::format_utils::SpaceSepFormatter;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UCISearchControl {
    pub search_moves: Vec<UCIMove>,
    pub mate: Option<u32>,
    pub depth: Option<u32>,
    pub nodes: Option<u64>,
}

impl Display for UCISearchControl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut formatter = SpaceSepFormatter::new(f);

        if !self.search_moves.is_empty() {
            formatter.push_str("searchmoves")?;

            for m in self.search_moves.iter() {
                formatter.push(m);
            }
        }

        formatter.push_option("mate", self.mate)?;
        formatter.push_option("depth", self.depth)?;
        formatter.push_option("nodes", self.nodes)?;

        Ok(())
    }
}
