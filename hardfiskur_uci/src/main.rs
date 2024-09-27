use std::{io::stdin, str::FromStr};

use hardfiskur_uci::UCIMessage;

fn main() {
    loop {
        let mut s = String::new();
        stdin().read_line(&mut s).unwrap();

        let parsed = UCIMessage::from_str(s.trim_ascii_end());
        match parsed {
            Ok(msg) => println!("{msg:#?}"),
            Err(e) => println!("{e}"),
        }
    }
}
