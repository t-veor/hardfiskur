use std::{fmt::Display, time::Duration};

use crate::format_utils::SpaceSepFormatter;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UCITimeControl {
    Infinite,
    MoveTime(Duration),
    TimeLeft {
        white_time: Option<Duration>,
        black_time: Option<Duration>,
        white_increment: Option<Duration>,
        black_increment: Option<Duration>,
        moves_to_go: Option<u32>,
    },
    Ponder,
}

impl Display for UCITimeControl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UCITimeControl::Ponder => write!(f, "ponder"),

            UCITimeControl::Infinite => write!(f, "infinite"),

            UCITimeControl::TimeLeft {
                white_time,
                black_time,
                white_increment,
                black_increment,
                moves_to_go,
            } => {
                let mut formatter = SpaceSepFormatter::new(f);

                formatter.push_option("wtime", white_time.map(|d| d.as_millis()))?;
                formatter.push_option("btime", black_time.map(|d| d.as_millis()))?;
                formatter.push_option("winc", white_increment.map(|d| d.as_millis()))?;
                formatter.push_option("binc", black_increment.map(|d| d.as_millis()))?;
                formatter.push_option("movestogo", *moves_to_go)?;

                Ok(())
            }

            UCITimeControl::MoveTime(duration) => write!(f, "{}", duration.as_millis()),
        }
    }
}
