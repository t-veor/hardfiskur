use hardfiskur_core::{
    board::{Bitboard, Board, Color, Piece, PieceType, Square},
    move_gen::{self, lookups::Lookups},
};

use crate::evaluation::piece_tables::material_score;

/// Static Exchange Evaluation (SEE) implementation.
/// See https://www.chessprogramming.org/Static_Exchange_Evaluation, and this
/// specific implementation:
/// https://www.chessprogramming.org/SEE_-_The_Swap_Algorithm
///
/// This implementation does not consider absolute pins when it plays its
/// captures.
pub struct Seer<'a> {
    board: &'a Board,
    lookups: &'static Lookups,

    occupied: Bitboard,
    diagonal_attackers: Bitboard,
    orthogonal_attackers: Bitboard,
}

impl<'a> Seer<'a> {
    pub fn new(board: &'a Board) -> Self {
        Self {
            board,
            lookups: Lookups::get_instance(),

            occupied: board.get_occupied_bitboard(),
            diagonal_attackers: Self::diagonal_pieces(board),
            orthogonal_attackers: Self::orthogonal_pieces(board),
        }
    }

    fn diagonal_pieces(board: &Board) -> Bitboard {
        board.get_bitboard_for_piece_type(PieceType::Bishop)
            | board.get_bitboard_for_piece_type(PieceType::Queen)
    }

    fn orthogonal_pieces(board: &Board) -> Bitboard {
        board.get_bitboard_for_piece_type(PieceType::Rook)
            | board.get_bitboard_for_piece_type(PieceType::Queen)
    }

    pub fn see(
        &self,
        from_square: Square,
        mut attacker: Piece,
        to_square: Square,
        target: Piece,
    ) -> i32 {
        let mut gain = Vec::with_capacity(32);

        let mut attacker_bb = Bitboard::from_square(from_square);
        let mut occupied = self.occupied;
        let mut attackers_and_defenders =
            move_gen::attackers_on(self.board.repr(), occupied, to_square, self.lookups);

        // Values in the gain array initially are the potential gain for the
        // opponent supposing the current piece is en-prise.
        // So gain[0] = value of the target piece
        gain.push(material_score(target.piece_type()));
        loop {
            // Then, gain[i] = value of taking on the target square with the
            // current attacker for the opponent, assuming the attacker can also
            // be taken en-prise.
            gain.push(material_score(attacker.piece_type()) - gain.last().unwrap());

            attackers_and_defenders ^= attacker_bb;
            occupied ^= attacker_bb;

            // The capture may reveal a new sliding attacker, add it to the
            // attackers_and_defenders bitboard
            if [PieceType::Pawn, PieceType::Bishop, PieceType::Queen]
                .contains(&attacker.piece_type())
            {
                attackers_and_defenders |=
                    self.lookups.get_bishop_attacks(occupied, to_square) & self.diagonal_attackers;
            }
            if [PieceType::Rook, PieceType::Queen].contains(&attacker.piece_type()) {
                attackers_and_defenders |=
                    self.lookups.get_rook_attacks(occupied, to_square) & self.orthogonal_attackers;
            }
            // Note that diagonal/orthogonal_attackers don't change as pieces
            // are exchanged off, so we need to mask them with the occupied mask
            // to remove pieces that are actually already gone
            attackers_and_defenders &= occupied;

            match self.get_least_valuable_piece(attackers_and_defenders, attacker.color().flip()) {
                Some((next_attacker_bb, next_attacker)) => {
                    attacker_bb = next_attacker_bb;
                    attacker = next_attacker;
                }
                None => break,
            }
        }

        // Run "minimax" on the resulting branching-factor-1 tree.
        // (Note gain.len() >= 2.)
        for i in (1..gain.len() - 1).rev() {
            gain[i - 1] = -std::cmp::max(-gain[i - 1], gain[i]);
        }

        gain[0]
    }

    fn get_least_valuable_piece(
        &self,
        attackers_and_defenders: Bitboard,
        color: Color,
    ) -> Option<(Bitboard, Piece)> {
        for piece_type in PieceType::ALL {
            let piece = piece_type.with_color(color);
            let subset = attackers_and_defenders & self.board.repr()[piece];
            if subset.has_piece() {
                return Some((subset.isolate_lsb(), piece));
            }
        }

        None
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use pretty_assertions::assert_eq;

    #[derive(Debug)]
    struct TestCase {
        fen: &'static str,
        from: Square,
        to: Square,
        expected_value: i32,
    }

    macro_rules! testcase {
        ($fen:expr, $from:ident, $to:ident, $( $expected_pieces:tt )* ) => {
            TestCase {
                fen: $fen,
                from: Square::$from,
                to: Square::$to,
                expected_value: expected_value!($( $expected_pieces )*),
            }
        };
    }

    macro_rules! expected_value {
        () => { 0 };
        (0) => { 0 };
        ($piece_type:ident $( $rest:tt )*) => {
            material_score(PieceType::$piece_type) + expected_value!($($rest)*)
        };
        (+ $piece_type:ident $( $rest:tt ) *) => {
            material_score(PieceType::$piece_type) + expected_value!($($rest)*)
        };
        (- $piece_type:ident $( $rest:tt ) *) => {
            -material_score(PieceType::$piece_type) + expected_value!($($rest)*)
        };
    }

    const TEST_CASES: &[TestCase] = &[
        testcase!("1k1r4/1pp4p/p7/4p3/8/P5P1/1PP4P/2K1R3 w - - 0 1", E1, E5, P),
        testcase!(
            "1k1r3q/1ppn3p/p4b2/4p3/8/P2N2P1/1PP1R1BP/2K1Q3 w - - 0 1",
            D3,
            E5,
            P - N
        ),
        testcase!(
            "7k/1pp4p/p1pb4/6q1/3P1pRr/2P4P/PP1Br1P1/5RKN w - - 0 1",
            F1,
            F4,
            P - R + B
        ),
        testcase!(
            "5rk1/1pp2q1p/p1pb4/8/3P1NP1/2P5/1P1BQ1P1/5RK1 b - - 0 1",
            D6,
            F4,
            N - B
        ),
        testcase!(
            "4R3/2r2rp1/5bk1/1p1p4/p3PRP1/P1B2P2/1PB5/2K5 b - - 0 1",
            D5,
            E4,
            0
        ),
        testcase!(
            "2r4k/2r2ppp/3n4/8/2p5/2RPn3/2Q5/2R2B1K w - - 0 1",
            C3,
            C4,
            N + N - B + R - Q
        ),
    ];

    #[test]
    fn run_test_cases() {
        for TestCase {
            fen,
            from,
            to,
            expected_value,
        } in TEST_CASES
        {
            let board = Board::try_parse_fen(fen).unwrap();
            let attacker = board.get_piece(*from).unwrap();
            let target = board.get_piece(*to).unwrap();

            let seer = Seer::new(&board);
            let see_value = seer.see(*from, attacker, *to, target);
            assert_eq!(
                see_value, *expected_value,
                "Incorrect SEE value received in position {} with move {}{}",
                fen, from, to
            );
        }
    }
}
