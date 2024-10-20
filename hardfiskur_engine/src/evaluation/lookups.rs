use hardfiskur_core::board::Bitboard;

const fn passed_pawn_masks() -> [[Bitboard; 64]; 2] {
    let mut masks = [[Bitboard::EMPTY; 64]; 2];

    let mut sq = 0;
    while sq < 64 {
        // White mask
        if sq < 56 {
            let forward = Bitboard(Bitboard::A_FILE.0 << (sq + 8));
            masks[0][sq] = forward.or(forward.step_east()).or(forward.step_west());
        }

        // Black mask
        if sq >= 8 {
            let backward = Bitboard(Bitboard::H_FILE.0 >> (63 - sq + 8));
            masks[1][sq] = backward.or(backward.step_east()).or(backward.step_west());
        }

        sq += 1;
    }

    masks
}

// by color, then square
pub const PASSED_PAWN_MASKS: [[Bitboard; 64]; 2] = passed_pawn_masks();

// . . . . . . . .
// . . . . . . . .
// . . . . . . . .
// . . . . . . . .
// . . . . . . . .
// . . . . . . . .
// # # . . . . # #
// # # # . # . # #
pub const SENSIBLE_KING_MASKS: [Bitboard; 2] = [Bitboard(0xC3D7), Bitboard(0xC3D7).flip_vertical()];

// by color, then by queenside (0) / kingside (1)
pub const PAWN_SHIELD_CLOSE_MASKS: [[Bitboard; 2]; 2] = [
    [
        // . . . . . . . .
        // . . . . . . . .
        // . . . . . . . .
        // . . . . . . . .
        // . . . . . . . .
        // . . . . . . . .
        // # # # . . . . .
        // . . . . . . . .
        Bitboard(0x700),
        // . . . . . . . .
        // . . . . . . . .
        // . . . . . . . .
        // . . . . . . . .
        // . . . . . . . .
        // . . . . . . . .
        // . . . . . # # #
        // . . . . . . . .
        Bitboard(0xE000),
    ],
    [
        Bitboard(0x700).flip_vertical(),
        Bitboard(0xE000).flip_vertical(),
    ],
];

// by color, then by queenside (0) / kingside (1)
pub const PAWN_SHIELD_FAR_MASKS: [[Bitboard; 2]; 2] = [
    [
        // . . . . . . . .
        // . . . . . . . .
        // . . . . . . . .
        // . . . . . . . .
        // . . . . . . . .
        // # # # . . . . .
        // . . . . . . . .
        // . . . . . . . .
        Bitboard(0x70000),
        // . . . . . . . .
        // . . . . . . . .
        // . . . . . . . .
        // . . . . . . . .
        // . . . . . . . .
        // . . . . . # # #
        // . . . . . . . .
        // . . . . . . . .
        Bitboard(0xE00000),
    ],
    [
        Bitboard(0x70000).flip_vertical(),
        Bitboard(0xE00000).flip_vertical(),
    ],
];

#[cfg(test)]
mod test {
    use super::*;

    use hardfiskur_core::board::Square;
    use pretty_assertions::assert_eq;

    #[test]
    fn white_passed_pawn_mask_e2() {
        let expected = "
        . . . # # # . .
        . . . # # # . .
        . . . # # # . .
        . . . # # # . .
        . . . # # # . .
        . . . # # # . .
        . . . . . . . .
        . . . . . . . .
        "
        .parse()
        .unwrap();

        assert_eq!(PASSED_PAWN_MASKS[0][Square::E2.index()], expected)
    }

    #[test]
    fn white_passed_pawn_mask_a4() {
        let expected = "
        # # . . . . . .
        # # . . . . . .
        # # . . . . . .
        # # . . . . . .
        . . . . . . . .
        . . . . . . . .
        . . . . . . . .
        . . . . . . . .
        "
        .parse()
        .unwrap();

        assert_eq!(PASSED_PAWN_MASKS[0][Square::A4.index()], expected)
    }

    #[test]
    fn white_passed_pawn_mask_h3() {
        let expected = "
        . . . . . . # #
        . . . . . . # #
        . . . . . . # #
        . . . . . . # #
        . . . . . . # #
        . . . . . . . .
        . . . . . . . .
        . . . . . . . .
        "
        .parse()
        .unwrap();

        assert_eq!(PASSED_PAWN_MASKS[0][Square::H3.index()], expected)
    }

    #[test]
    fn white_passed_pawn_mask_a1() {
        let expected = "
        # # . . . . . .
        # # . . . . . .
        # # . . . . . .
        # # . . . . . .
        # # . . . . . .
        # # . . . . . .
        # # . . . . . .
        . . . . . . . .
        "
        .parse()
        .unwrap();

        assert_eq!(PASSED_PAWN_MASKS[0][Square::A1.index()], expected)
    }

    #[test]
    fn white_passed_pawn_mask_h7() {
        let expected = "
        . . . . . . # #
        . . . . . . . .
        . . . . . . . .
        . . . . . . . .
        . . . . . . . .
        . . . . . . . .
        . . . . . . . .
        . . . . . . . .
        "
        .parse()
        .unwrap();

        assert_eq!(PASSED_PAWN_MASKS[0][Square::H7.index()], expected)
    }

    #[test]
    fn white_passed_pawn_mask_backrank() {
        for file in 0..8 {
            assert_eq!(
                PASSED_PAWN_MASKS[0][Square::new(7, file).unwrap().index()],
                Bitboard::EMPTY
            )
        }
    }

    #[test]
    fn black_passed_pawn_mask_e7() {
        let expected = "
        . . . . . . . .
        . . . . . . . .
        . . . # # # . .
        . . . # # # . .
        . . . # # # . .
        . . . # # # . .
        . . . # # # . .
        . . . # # # . .
        "
        .parse()
        .unwrap();

        assert_eq!(PASSED_PAWN_MASKS[1][Square::E7.index()], expected)
    }

    #[test]
    fn black_passed_pawn_mask_a4() {
        let expected = "
        . . . . . . . .
        . . . . . . . .
        . . . . . . . .
        . . . . . . . .
        . . . . . . . .
        # # . . . . . .
        # # . . . . . .
        # # . . . . . .
        "
        .parse()
        .unwrap();

        assert_eq!(PASSED_PAWN_MASKS[1][Square::A4.index()], expected)
    }

    #[test]
    fn black_passed_pawn_mask_h3() {
        let expected = "
        . . . . . . . .
        . . . . . . . .
        . . . . . . . .
        . . . . . . . .
        . . . . . . . .
        . . . . . . . .
        . . . . . . # #
        . . . . . . # #
        "
        .parse()
        .unwrap();

        assert_eq!(PASSED_PAWN_MASKS[1][Square::H3.index()], expected)
    }

    #[test]
    fn black_passed_pawn_mask_a2() {
        let expected = "
        . . . . . . . .
        . . . . . . . .
        . . . . . . . .
        . . . . . . . .
        . . . . . . . .
        . . . . . . . .
        . . . . . . . .
        # # . . . . . .
        "
        .parse()
        .unwrap();

        assert_eq!(PASSED_PAWN_MASKS[1][Square::A2.index()], expected)
    }

    #[test]
    fn white_passed_pawn_mask_h8() {
        let expected = "
        . . . . . . . .
        . . . . . . # #
        . . . . . . # #
        . . . . . . # #
        . . . . . . # #
        . . . . . . # #
        . . . . . . # #
        . . . . . . # #
        "
        .parse()
        .unwrap();

        assert_eq!(PASSED_PAWN_MASKS[1][Square::H8.index()], expected)
    }

    #[test]
    fn black_passed_pawn_mask_backrank() {
        for file in 0..8 {
            assert_eq!(
                PASSED_PAWN_MASKS[1][Square::new(0, file).unwrap().index()],
                Bitboard::EMPTY
            )
        }
    }

    #[test]
    fn sensible_king_masks() {
        assert_eq!(
            SENSIBLE_KING_MASKS[0],
            "
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                # # . . . . # #
                # # # . # . # #
            "
            .parse()
            .unwrap()
        );

        assert_eq!(
            SENSIBLE_KING_MASKS[1],
            "
                # # # . # . # #
                # # . . . . # #
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
}
