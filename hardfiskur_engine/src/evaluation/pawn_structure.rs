use hardfiskur_core::{
    board::{Bitboard, Board, Piece},
    move_gen,
};

use super::{lookups::PASSED_PAWN_MASKS, template_params::ColorParam};

#[derive(Debug, Clone)]
pub struct PawnStructure {
    pub pawns: [Bitboard; 2],
    pub pawn_attacks: [Bitboard; 2],
    pub passed_pawns: [Bitboard; 2],
    pub semi_open_files: [Bitboard; 2],
}

impl PawnStructure {
    pub fn new(board: &Board) -> Self {
        let white_pawns = board.repr()[Piece::WHITE_PAWN];
        let black_pawns = board.repr()[Piece::BLACK_PAWN];

        let white_pawn_attacks = move_gen::white_pawn_attacks(white_pawns);
        let black_pawn_attacks = move_gen::black_pawn_attacks(black_pawns);

        let white_passed_pawns = white_pawns
            .squares()
            .filter(|sq| (PASSED_PAWN_MASKS[0][sq.index()] & black_pawns).is_empty())
            .collect();

        let black_passed_pawns = black_pawns
            .squares()
            .filter(|sq| (PASSED_PAWN_MASKS[1][sq.index()] & white_pawns).is_empty())
            .collect();

        let white_semi_open_files = (0..8)
            .map(Bitboard::file_mask)
            .filter(|&file| (file & white_pawns).is_empty())
            .fold(Bitboard::EMPTY, |acc, bb| acc | bb);
        let black_semi_open_files = (0..8)
            .map(Bitboard::file_mask)
            .filter(|&file| (file & black_pawns).is_empty())
            .fold(Bitboard::EMPTY, |acc, bb| acc | bb);

        Self {
            pawns: [white_pawns, black_pawns],
            pawn_attacks: [white_pawn_attacks, black_pawn_attacks],
            passed_pawns: [white_passed_pawns, black_passed_pawns],
            semi_open_files: [white_semi_open_files, black_semi_open_files],
        }
    }

    pub fn isolated_pawns<C: ColorParam>(&self) -> Bitboard {
        let pawns = self.pawns[C::INDEX];
        let semi_open_files = self.semi_open_files[C::INDEX];

        pawns
            & (semi_open_files.step_west() | Bitboard::H_FILE)
            & (semi_open_files.step_east() | Bitboard::A_FILE)
    }
}

#[cfg(test)]
mod test {
    use crate::evaluation::template_params::{Black, White};

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn passed_pawns_1() {
        let board = Board::try_parse_fen("4k3/8/4p2p/5p2/1P6/P2P3P/8/4K3 w - - 0 1").unwrap();
        let pawns = PawnStructure::new(&board);

        assert_eq!(
            pawns.passed_pawns[0],
            "
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . # . . . . . .
                # . . . . . . .
                . . . . . . . .
                . . . . . . . .
            "
            .parse()
            .unwrap()
        );

        assert_eq!(
            pawns.passed_pawns[1],
            "
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . # . .
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
            "
            .parse()
            .unwrap()
        );
    }

    #[test]
    fn passed_pawns_2() {
        let board = Board::try_parse_fen("4k3/8/7p/1P2Pp1P/2Pp1PP1/8/8/4K3 w - - 0 1").unwrap();
        let pawns = PawnStructure::new(&board);

        assert_eq!(
            pawns.passed_pawns[0],
            "
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . # . . # . . .
                . . # . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
            "
            .parse()
            .unwrap()
        );

        assert_eq!(
            pawns.passed_pawns[1],
            "
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . # . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
            "
            .parse()
            .unwrap()
        );
    }

    #[test]
    fn semi_open_files() {
        let board = Board::try_parse_fen("4k3/p3p3/1p6/4Pp1p/8/2P5/PP3PP1/4K3 w - - 0 1").unwrap();
        let pawns = PawnStructure::new(&board);

        assert_eq!(
            pawns.semi_open_files[0],
            "
                . . . # . . . #
                . . . # . . . #
                . . . # . . . #
                . . . # . . . #
                . . . # . . . #
                . . . # . . . #
                . . . # . . . #
                . . . # . . . #
            "
            .parse()
            .unwrap()
        );

        assert_eq!(
            pawns.semi_open_files[1],
            "
                . . # # . . # .
                . . # # . . # .
                . . # # . . # .
                . . # # . . # .
                . . # # . . # .
                . . # # . . # .
                . . # # . . # .
                . . # # . . # .
            "
            .parse()
            .unwrap()
        );
    }

    #[test]
    fn isolated_pawns() {
        let board =
            Board::try_parse_fen("4k3/p1p3p1/3p3p/1P5P/1PP1P1P1/8/8/4K3 w - - 0 1").unwrap();
        let pawns = PawnStructure::new(&board);

        let white_isolated_pawns = pawns.isolated_pawns::<White>();
        let black_isolated_pawns = pawns.isolated_pawns::<Black>();

        assert_eq!(
            white_isolated_pawns,
            "
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . # . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
            "
            .parse()
            .unwrap()
        );

        assert_eq!(
            black_isolated_pawns,
            "
                . . . . . . . .
                # . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
            "
            .parse()
            .unwrap()
        );
    }

    #[test]
    fn isolated_pawns_edge() {
        let board = Board::try_parse_fen("4k3/p7/1p5p/8/P7/6PP/8/4K3 w - - 0 1").unwrap();
        let pawns = PawnStructure::new(&board);

        let white_isolated_pawns = pawns.isolated_pawns::<White>();
        let black_isolated_pawns = pawns.isolated_pawns::<Black>();

        assert_eq!(
            white_isolated_pawns,
            "
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                # . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
            "
            .parse()
            .unwrap()
        );

        assert_eq!(
            black_isolated_pawns,
            "
                . . . . . . . .
                . . . . . . . .
                . . . . . . . #
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
            "
            .parse()
            .unwrap()
        );
    }
}
