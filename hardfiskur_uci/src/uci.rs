use std::{io::stdin, str::FromStr, time::Duration};

use hardfiskur_core::board::{Board, Color, UCIMove};
use hardfiskur_engine::{
    search_limits::SearchLimits,
    search_result::{SearchInfo, SearchResult},
    Engine, SearchReporter,
};
use hardfiskur_uci::{UCIMessage, UCIOptionConfig, UCIPosition, UCIPositionBase, UCITimeControl};

fn version_string() -> String {
    let rev = option_env!("VERGEN_GIT_DESCRIBE").unwrap_or("unknown");
    let dirty = if option_env!("VERGEN_GIT_DIRTY") == Some("true") {
        "-DIRTY"
    } else {
        ""
    };

    format!("rev {rev}{dirty}")
}

fn read_message() -> Option<UCIMessage> {
    let mut s = String::new();
    match stdin().read_line(&mut s) {
        Ok(0) => {
            // Reached EOF.
            Some(UCIMessage::Quit)
        }
        Ok(_) => UCIMessage::from_str(&s).ok(),
        Err(e) => {
            panic!("Error reading from stdin: {e}")
        }
    }
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

        Some(UCITimeControl::Infinite) => return Duration::MAX,

        _ => (),
    }

    // Default 2s
    Duration::from_secs(2)
}

struct UCIReporter;
impl SearchReporter for UCIReporter {
    fn receive_search_info(&self, info: SearchInfo) {
        println!("{}", UCIMessage::Info(info.into()));
    }

    fn search_complete(&self, result: SearchResult) {
        let SearchResult {
            best_move, info, ..
        } = result;

        println!("{}", UCIMessage::Info(info.into()));

        let best_move = match best_move {
            Some(x) => x,
            None => {
                eprintln!("Engine did not return a move!");
                return;
            }
        };

        println!("{}", UCIMessage::best_move(best_move.into()))
    }
}

fn uci_options() -> Vec<UCIOptionConfig> {
    vec![
        UCIOptionConfig::Spin {
            name: "Hash".into(),
            default: Some(32),
            min: Some(1),
            max: Some(4096),
        },
        UCIOptionConfig::Spin {
            name: "Threads".into(),
            default: Some(1),
            min: Some(1),
            max: Some(1),
        },
    ]
}

fn handle_option(engine: &mut Engine, option_name: &str, option_value: Option<&str>) {
    match option_name {
        "Hash" => {
            let value = match option_value.and_then(|x| x.parse().ok()) {
                Some(x) => x,
                None => {
                    eprintln!("Could not parse {option_value:?} as usize");
                    return;
                }
            };

            if !(1..=4096).contains(&value) {
                eprintln!("Invalid value for Hash: {value} (min=1, max=4096)");
                return;
            }

            engine.set_tt_size(value);
        }
        _ => (),
    }
}

pub fn main_loop(engine: &mut Engine) {
    let mut current_board = Board::starting_position();

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
                    UCIMessage::id_name(&format!("Harðfiskur ({})", version_string()))
                );
                println!("{}", UCIMessage::id_author("Tyler Zhang"));

                for option in uci_options() {
                    println!("{option}");
                }

                println!("{}", UCIMessage::UCIOk);
            }

            UCIMessage::SetOption { name, value } => {
                handle_option(engine, &name, value.as_ref().map(|x| x.as_str()))
            }

            UCIMessage::UCINewGame => {
                current_board = Board::starting_position();
                engine.new_game();
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
                        .and_then(|d| d.try_into().ok())
                        .unwrap_or(i16::MAX),
                };

                engine.start_search(&current_board, search_limits, UCIReporter);
            }

            UCIMessage::Stop => engine.abort_search(),

            UCIMessage::D => {
                println!("{current_board}");
                println!("FEN: {}", current_board.fen());
                println!("{:?}", current_board.zobrist_hash());
            }

            UCIMessage::TTEntry => {
                println!("TT entry for {:?}:", current_board.zobrist_hash());
                match engine.get_tt_entry(&current_board) {
                    Some(entry) => println!("{entry}"),
                    None => println!("<none>"),
                }
            }

            UCIMessage::MakeMove(m) => {
                let m = m.or_else(|| {
                    let entry = engine.get_tt_entry(&current_board);
                    entry.and_then(|m| m.best_move).map(|m| {
                        let m = m.into();
                        println!("Using best move from TT: {m}");
                        m
                    })
                });

                if let Some(m) = m {
                    let pushed_move = current_board.push_move(m.from, m.to, m.promotion);
                    if pushed_move.is_none() {
                        println!("Move {m} was invalid");
                    }
                } else {
                    println!("No best move found in TT");
                }
            }

            UCIMessage::UndoMove => {
                if let Some(m) = current_board.pop_move() {
                    println!("Undid move {}", UCIMove::from(m));
                }
            }

            UCIMessage::GetPV => {
                let pv = engine.get_pv(&current_board);
                print!("PV:");
                for m in pv {
                    print!(" {}", UCIMove::from(m));
                }
                println!();
            }

            UCIMessage::Eval => println!("{}", engine.debug_eval(&current_board)),

            UCIMessage::Bench => {
                let (nodes, time) = engine.bench();

                let nps = nodes * 1000 / time.as_millis() as u64;

                println!(
                    "info string nodes {nodes} time {} nps {nps}",
                    time.as_millis()
                );
            }

            // ignore all other messages
            _ => (),
        }
    }
}
