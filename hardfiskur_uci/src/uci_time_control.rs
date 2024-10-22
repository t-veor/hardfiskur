use std::{fmt::Display, time::Duration};

use hardfiskur_core::board::Color;
use hardfiskur_engine::search_limits::TimeControls;

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

impl UCITimeControl {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn from_raw(
        ponder: bool,
        white_time: Option<Duration>,
        black_time: Option<Duration>,
        white_increment: Option<Duration>,
        black_increment: Option<Duration>,
        moves_to_go: Option<u32>,
        move_time: Option<Duration>,
        infinite: bool,
    ) -> Option<Self> {
        if infinite {
            Some(UCITimeControl::Infinite)
        } else if let Some(move_time) = move_time {
            Some(UCITimeControl::MoveTime(move_time))
        } else if white_time.is_some()
            || black_time.is_some()
            || white_increment.is_some()
            || black_increment.is_some()
            || moves_to_go.is_some()
        {
            Some(UCITimeControl::TimeLeft {
                white_time,
                black_time,
                white_increment,
                black_increment,
                moves_to_go,
            })
        } else if ponder {
            Some(UCITimeControl::Ponder)
        } else {
            None
        }
    }

    pub fn as_time_controls(self, to_move: Color) -> TimeControls {
        match self {
            UCITimeControl::Infinite => TimeControls::Infinite,
            UCITimeControl::MoveTime(duration) => TimeControls::FixedMoveTime(duration),
            UCITimeControl::TimeLeft {
                white_time,
                black_time,
                white_increment,
                black_increment,
                moves_to_go,
            } => {
                let (remaining, increment) = match to_move {
                    Color::White => (white_time, white_increment),
                    Color::Black => (black_time, black_increment),
                };

                let remaining = remaining.unwrap_or(Duration::ZERO);
                let increment = increment.unwrap_or(Duration::ZERO);

                match moves_to_go {
                    Some(moves_to_go) => TimeControls::Cyclic {
                        remaining,
                        increment,
                        moves_to_go,
                    },
                    None => TimeControls::FischerTime {
                        remaining,
                        increment,
                    },
                }
            }
            UCITimeControl::Ponder => TimeControls::Infinite,
        }
    }
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
