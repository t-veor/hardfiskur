use std::{io::stdin, str::FromStr};

use uci_message::UCIMessage;

mod format_utils;
mod parse_utils;
mod uci_info;
mod uci_message;
mod uci_option_config;
mod uci_position;
mod uci_search_control;
mod uci_time_control;

fn main() {
    loop {
        let mut s = String::new();
        stdin().read_line(&mut s).unwrap();

        let parsed = UCIMessage::from_str(&s);
        match parsed {
            Ok(msg) => println!("{msg:#?}"),
            Err(e) => println!("{e}"),
        }
    }
}
