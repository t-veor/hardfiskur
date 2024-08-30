use crate::{board::Board, move_gen::MoveVec};

pub fn perft(board: &mut Board, depth: usize) -> u64 {
    if depth == 0 {
        return 1;
    }

    let mut moves = MoveVec::new();
    let mut nodes = 0;

    board.legal_moves_ex(Default::default(), &mut moves);
    if depth == 1 {
        return moves.len() as _;
    }

    for m in moves.into_iter() {
        board.push_move_unchecked(m);
        nodes += perft(board, depth - 1);
        board.pop_move().unwrap();
    }

    nodes
}

#[cfg(test)]
mod test {
    use crate::board::STARTING_POSITION_FEN;
    use pretty_assertions::assert_eq;

    use super::*;

    const KIWIPETE_FEN: &str =
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";

    const TEST_3_FEN: &str = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1";

    const TEST_4_FEN: &str = "r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1";

    const TEST_5_FEN: &str = "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8";

    const TEST_6_FEN: &str =
        "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10";

    fn test_perft(fen: &str, expected_nodes: &[u64]) {
        let mut board = Board::try_parse_fen(fen).expect("Invalid FEN");
        let mut got = Vec::new();

        for i in 0..expected_nodes.len() {
            let nodes = perft(&mut board, i);
            got.push(nodes);
        }

        assert_eq!(got, expected_nodes);
    }

    #[test]
    fn test_starting_position() {
        const EXPECTED: &[u64] = &[1, 20, 400, 8_902, 197_281, 4_865_609];
        test_perft(STARTING_POSITION_FEN, EXPECTED);
    }

    #[test]
    fn test_kiwipete() {
        const EXPECTED: &[u64] = &[1, 48, 2039, 97_862, 4_085_603];
        test_perft(KIWIPETE_FEN, EXPECTED);
    }

    #[test]
    fn test_3() {
        const EXPECTED: &[u64] = &[1, 14, 191, 2_812, 43_238, 674_624];
        test_perft(TEST_3_FEN, EXPECTED);
    }

    #[test]
    fn test_4() {
        const EXPECTED: &[u64] = &[1, 6, 264, 9_467, 422_333];
        test_perft(TEST_4_FEN, EXPECTED);
    }

    #[test]
    fn test_5() {
        const EXPECTED: &[u64] = &[1, 44, 1_486, 62_379, 2_103_487];
        test_perft(TEST_5_FEN, EXPECTED);
    }

    #[test]
    fn test_6() {
        const EXPECTED: &[u64] = &[1, 46, 2_079, 89_890, 3_894_594];
        test_perft(TEST_6_FEN, EXPECTED);
    }
}
