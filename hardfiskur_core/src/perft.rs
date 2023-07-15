use crate::{board::Board, move_gen::MoveVec};

pub fn perft(board: &mut Board, depth: usize) -> u64 {
    if depth == 0 {
        return 1;
    }

    let mut moves = MoveVec::new();
    let mut nodes = 0;

    board.legal_moves_ex(Default::default(), &mut moves);
    for m in moves.into_iter() {
        board.push_move_unchecked(m);
        nodes += perft(board, depth - 1);
        board.pop_move().unwrap();
    }

    nodes
}
