use std::{
    io::stdin,
    str::FromStr,
    sync::{Arc, Mutex},
    time::Duration,
};

use hardfiskur_core::board::{Board, Color};
use hardfiskur_engine::{
    search::iterative_deepening_search, transposition_table::TranspositionTable,
};
use hardfiskur_uci::{UCIInfo, UCIMessage, UCIPosition, UCIPositionBase, UCITimeControl};
use threadpool::ThreadPool;

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

        _ => (),
    }

    // Default 2s
    Duration::from_secs(2)
}

fn main() {
    let mut current_board = Board::starting_position();
    let threadpool = ThreadPool::new(1);

    let transposition_table = Arc::new(Mutex::new(TranspositionTable::new(64)));

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
                search_control: _,
            } => {
                let mut board = current_board.clone();
                let allocated_time = simple_time_allocation(board.to_move(), time_control.as_ref());

                let transposition_table = transposition_table.clone();

                threadpool.execute(move || {
                    let mut transposition_table = transposition_table.lock().unwrap();
                    if let (score, Some(m), stats) = iterative_deepening_search(
                        &mut board,
                        allocated_time,
                        &mut transposition_table,
                    ) {
                        let elapsed = stats.search_started.elapsed();
                        println!(
                            "{}",
                            UCIMessage::Info(UCIInfo {
                                depth: Some(stats.depth),
                                time: Some(elapsed),
                                nodes: Some(stats.nodes_searched),
                                score: Some(score.into()),
                                ..Default::default()
                            })
                        );
                        println!("{}", UCIMessage::best_move(m.into()))
                    }
                });
            }

            // TODO
            UCIMessage::Stop => (),

            // ignore all other messages
            _ => (),
        }
    }
}
