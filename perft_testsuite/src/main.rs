use std::{process::ExitCode, time::Instant};

use hardfiskur_core::{board::Board, perft::perft};

#[derive(Debug, Clone)]
struct TestCase {
    fen: String,
    expected_results: Vec<u64>,
}

fn parse_test_cases() -> Vec<TestCase> {
    include_str!("perftsuite.txt")
        .lines()
        .map(|line| {
            let (fen, results) = line.split_once(" ;").unwrap();
            let expected_results = results
                .split(" ;")
                .map(|val| {
                    let (_, nodes) = val.split_once(' ').unwrap();
                    nodes.parse().unwrap()
                })
                .collect();

            TestCase {
                fen: fen.to_string(),
                expected_results,
            }
        })
        .collect()
}

fn run_test_case(id: usize, case: &TestCase) -> bool {
    println!("Test case {id}: {}", case.fen);
    let mut board = Board::try_parse_fen(&case.fen).unwrap();

    let mut failed = false;
    for (i, &expected) in case.expected_results.iter().enumerate() {
        let depth = i + 1;
        print!("Depth {depth}: Expected {expected}, ");
        let received = perft(&mut board, depth);
        print!("got {received}");

        if expected == received {
            println!();
        } else {
            failed = true;
            println!(" -- ERROR");
        }
    }

    !failed
}

fn main() -> ExitCode {
    let test_cases = parse_test_cases();
    let mut failed_cases = vec![];

    let start = Instant::now();
    for (i, case) in test_cases.iter().enumerate() {
        let id = i + 1;
        if !run_test_case(id, case) {
            failed_cases.push(id.to_string());
        }
    }
    let elapsed = start.elapsed();

    println!("Took {:?}.", elapsed);

    if failed_cases.is_empty() {
        println!("All test cases passed.");
        ExitCode::SUCCESS
    } else {
        println!("Failing cases: {}", failed_cases.join(", "));
        ExitCode::FAILURE
    }
}
