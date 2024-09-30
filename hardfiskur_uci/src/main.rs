use std::{io::stdin, str::FromStr};

use hardfiskur_core::board::Board;
use hardfiskur_engine::search::simple_search;
use hardfiskur_uci::{UCIMessage, UCIPosition, UCIPositionBase};
use threadpool::ThreadPool;

fn read_message() -> Option<UCIMessage> {
    let mut s = String::new();
    stdin().read_line(&mut s).ok()?;

    UCIMessage::from_str(&s).ok()
}

fn main() {
    let mut current_board = Board::starting_position();
    let threadpool = ThreadPool::new(1);

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
                time_control: _,
                search_control: _,
            } => {
                let mut board = current_board.clone();
                threadpool.execute(move || {
                    if let (score, Some(m)) = simple_search(&mut board) {
                        println!("{}", UCIMessage::info_score(score));
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
