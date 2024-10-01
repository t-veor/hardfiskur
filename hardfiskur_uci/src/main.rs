use std::{io::stdin, str::FromStr, time::Duration, u64};

use hardfiskur_core::board::{Board, Color};
use hardfiskur_engine::{search_limits::SearchLimits, search_result::SearchResult, Engine};
use hardfiskur_uci::{UCIInfo, UCIMessage, UCIPosition, UCIPositionBase, UCITimeControl};

fn read_message() -> Option<UCIMessage> {
    let mut s = String::new();
    stdin().read_line(&mut s).ok()?;

    UCIMessage::from_str(&s).ok()
}

fn simple_time_allocation(to_move: Color, time_control: Option<&UCITimeControl>) -> Duration {
    match time_control {
        Some(UCITimeControl::MoveTime(duration)) => {
            // Use move time minus 25ms
            return duration.saturating_sub(Duration::from_millis(25));
        }
        Some(UCITimeControl::TimeLeft {
            white_time,
            black_time,
            white_increment,
            black_increment,
            ..
        }) => {
            let (time_remaining, increment) = match to_move {
                Color::White => (white_time, white_increment),
                Color::Black => (black_time, black_increment),
            };

            if let Some(time_remaining) = time_remaining {
                let increment = increment.unwrap_or(Duration::ZERO);

                return *time_remaining / 20 + increment / 2;
            }
        }

        Some(UCITimeControl::Infinite) | Some(UCITimeControl::Ponder) => return Duration::MAX,

        _ => (),
    }

    // Default 2s
    Duration::from_secs(2)
}

fn main() {
    let mut current_board = Board::starting_position();
    let mut engine = Engine::new();

    'main_loop: loop {
        let command = match read_message() {
            Some(command) => command,
            None => {
                eprintln!("Could not parse UCI message");
                continue 'main_loop;
            }
        };

        match command {
            UCIMessage::Quit => return,

            UCIMessage::UCI => {
                println!(
                    "{}",
                    UCIMessage::id_name(&format!("Harðfiskur (rev {})", env!("GIT_HASH_SHORT")))
                );
                println!("{}", UCIMessage::id_author("Tyler Zhang"));
                println!("{}", UCIMessage::UCIOk);
            }

            UCIMessage::UCINewGame => engine.new_game(),

            UCIMessage::IsReady => {
                println!("{}", UCIMessage::ReadyOk);
            }

            UCIMessage::Position(UCIPosition { base, moves }) => {
                match base {
                    UCIPositionBase::StartPos => current_board = Board::starting_position(),
                    UCIPositionBase::Fen(fen) => {
                        current_board = match Board::try_parse_fen(&fen) {
                            Ok(board) => board,
                            Err(e) => {
                                eprintln!("Could not parse fen: {} ({e})", fen);
                                continue 'main_loop;
                            }
                        };
                    }
                }

                for m in moves {
                    if current_board.push_move(m.from, m.to, m.promotion).is_none() {
                        eprintln!("Invalid move received: {m}")
                    }
                }
            }

            UCIMessage::Go {
                time_control,
                search_control,
            } => {
                let allocated_time =
                    simple_time_allocation(current_board.to_move(), time_control.as_ref());

                let search_limits = SearchLimits {
                    allocated_time,
                    node_budget: search_control
                        .as_ref()
                        .and_then(|s| s.nodes)
                        .unwrap_or(u64::MAX),
                    depth: search_control
                        .as_ref()
                        .and_then(|s| s.depth)
                        .unwrap_or(u32::MAX),
                };

                engine.start_search(&current_board, search_limits, |result| {
                    let SearchResult {
                        score,
                        best_move,
                        stats,
                        elapsed,
                        ..
                    } = result;

                    let best_move = match best_move {
                        Some(m) => m,
                        None => {
                            eprintln!("Search did not return a move!");
                            return;
                        }
                    };

                    let nps = 1000 * stats.nodes_searched / (elapsed.as_millis() as u64).max(1);

                    println!(
                        "{}",
                        UCIMessage::Info(UCIInfo {
                            depth: Some(stats.depth),
                            time: Some(elapsed),
                            nodes: Some(stats.nodes_searched),
                            score: Some(score.into()),
                            tb_hits: Some(stats.tt_hits),
                            nps: Some(nps),
                            ..Default::default()
                        })
                    );
                    println!("{}", UCIMessage::best_move(best_move.into()))
                });
            }

            UCIMessage::Stop => engine.abort_search(),

            // ignore all other messages
            _ => (),
        }
    }
}
