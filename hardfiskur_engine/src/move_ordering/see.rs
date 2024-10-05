use hardfiskur_core::{
    board::{Bitboard, Board, Color, Piece, PieceType, Square},
    move_gen::{self, lookups::Lookups},
};

/// Static Exchange Evaluation (SEE) implementation.
/// See https://www.chessprogramming.org/Static_Exchange_Evaluation.
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
    // (unused), pawn, knight, bishop, rook, queen
    const SEE_VALUES: [i32; 7] = [0, 100, 400, 400, 650, 1200, 0];

    pub fn new(board: &'a Board) -> Self {
        Self {
            board,
            lookups: Lookups::get_instance(),

            occupied: board.get_occupied_bitboard(),
            diagonal_attackers: Self::diagonal_pieces(board),
            orthogonal_attackers: Self::orthogonal_pieces(board),
        }
    }

    fn value(piece: impl Into<PieceType>) -> i32 {
        Self::value_const(piece.into())
    }

    const fn value_const(piece_type: PieceType) -> i32 {
        Self::SEE_VALUES[piece_type as usize]
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
        attacker: Piece,
        to_square: Square,
        target: Piece,
        threshold: i32,
    ) -> bool {
        // First, simulate making the capture.

        let mut balance = Self::value(target) - threshold;
        if balance < 0 {
            return false;
        }

        balance -= Self::value(attacker);
        if balance >= 0 {
            return true;
        }

        let attacker_bb = Bitboard::from_square(from_square);
        let mut occupied = self.occupied ^ attacker_bb;
        let mut attackers_and_defenders =
            move_gen::attackers_on(self.board.repr(), occupied, to_square, self.lookups)
                ^ attacker_bb;

        let mut side_to_move = self.board.to_move().flip();

        loop {
            let (attacker_bb, attacker) =
                match self.get_least_valuable_piece(attackers_and_defenders, side_to_move) {
                    Some(x) => x,
                    None => break,
                };

            if attacker.is_king()
                && (attackers_and_defenders & self.board.repr()[side_to_move.flip()]).has_piece()
            {
                break;
            }

            // Make the capture
            occupied ^= attacker_bb;

            // Don't need this line because the attacker gets cleared off the
            // attackers_and_defenders bitboard by the final &= occupied mask at
            // the end of this loop
            // attackers_and_defenders ^= attacker_bb;

            side_to_move = side_to_move.flip();

            balance = -balance - Self::value(attacker);
            if balance >= 0 {
                break;
            }

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
        }

        side_to_move != self.board.to_move()
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
            Seer::value_const(PieceType::$piece_type) + expected_value!($($rest)*)
        };
        (+ $piece_type:ident $( $rest:tt ) *) => {
            Seer::value_const(PieceType::$piece_type) + expected_value!($($rest)*)
        };
        (- $piece_type:ident $( $rest:tt ) *) => {
            -Seer::value_const(PieceType::$piece_type) + expected_value!($($rest)*)
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
            let see_value = seer.see(*from, attacker, *to, target, 1);
            // assert_eq!(
            //     see_value, *expected_value,
            //     "Incorrect SEE value received in position {} with move {}{}",
            //     fen, from, to
            // );
        }
    }
}
