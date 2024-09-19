use std::time::{Duration, Instant};

use clap::Parser;
use hardfiskur_core::{
    board::{Board, Piece, UCIMove},
    perft::perft,
};

/// Perft tester for Harðfiskur.
///
/// Runs perft on the move generator for Harðfiskur to debug issues and check
/// its performance.
#[derive(Parser, Debug)]
struct Args {
    /// Starting position of the board.
    ///
    /// Expects either the string "startpos" or a valid position in
    /// Forsyth-Edwards Notation (FEN). Additional alterations to the position
    /// can be specified via the --moves option.
    #[arg(short, long, default_value = "startpos", value_parser = parse_position)]
    position: Board,

    /// Additional moves to play before running perft.
    ///
    /// Plays the specified moves on top of the position specified with
    /// `--position` before running perft. Can be used to quickly test
    /// variations on a position.
    ///
    /// Moves should consist of the starting square in algebraic notation
    /// followed by the ending square in algebraic notation, plus an optional
    /// promotion target as a lowercase FEN char, e.g. `d2d4`, `e2e1q`
    #[arg(short, long, num_args(0..))]
    moves: Vec<UCIMove>,

    /// Exact depth to search to.
    #[arg(short, long, value_parser = clap::value_parser!(u8).range(1..), default_value_t = 8)]
    depth: u8,

    /// Run in divide mode.
    ///
    /// When provided, will run perft down to the provided depth and list each
    /// possible move in the current position along with the number of nodes
    /// found under that move. This is useful for debugging errors by
    /// identifying the exact sequence of moves under which they occur.
    #[arg(long)]
    divide: bool,
}

fn parse_position(s: &str) -> Result<Board, String> {
    if s == "startpos" {
        Ok(Board::starting_position())
    } else {
        Board::try_parse_fen(s).map_err(|e| {
            format!("Expected `startpos` or a valid FEN string. FEN parsing error: {e}")
        })
    }
}

fn generic_perft(mut board: Board, max_depth: usize) {
    let mut total_time = Duration::ZERO;
    let mut last_depth_time = Duration::ZERO;
    let mut total_nodes = 0;

    for depth in 0..max_depth {
        let start_time = Instant::now();

        let nodes = perft(&mut board, depth);

        let time_taken = start_time.elapsed();

        println!(
            "Depth: {depth}\tNodes: {nodes}\tTime taken: {:.3}s",
            time_taken.as_secs_f64()
        );

        total_time += time_taken;
        last_depth_time = time_taken;
        total_nodes += nodes;
    }

    println!();

    let nodes_per_second = total_nodes as f64 / last_depth_time.as_secs_f64();

    println!(
        "Total nodes: {total_nodes}\tTotal time: {:.3}s\tNodes per second: {:.3}",
        total_time.as_secs_f64(),
        nodes_per_second
    );
}

fn specific_perft(mut board: Board, depth: usize) {
    assert!(depth >= 1);

    let legal_moves = board.legal_moves();

    let mut total_nodes = 0;
    for m in legal_moves {
        let move_spec = UCIMove {
            from: m.from_square(),
            to: m.to_square(),
            promotion: m.promotion().map(Piece::piece_type),
        };

        board.push_move_unchecked(m);
        let nodes = perft(&mut board, depth - 1);
        board.pop_move();

        total_nodes += nodes;
        println!("{move_spec}: {nodes}");
    }

    println!();
    println!("Nodes searched: {total_nodes}");
}

fn main() -> Result<(), String> {
    let Args {
        position,
        moves,
        depth,
        divide,
    } = Args::parse();

    let mut board = position;
    for move_spec in moves {
        if board
            .push_move(move_spec.from, move_spec.to, move_spec.promotion)
            .is_none()
        {
            return Err(format!(
                "Move `{move_spec}` is invalid to play in this position ({})",
                board.fen()
            ));
        }
    }

    if divide {
        specific_perft(board, depth as _)
    } else {
        generic_perft(board, depth as _);
    }

    Ok(())
}
