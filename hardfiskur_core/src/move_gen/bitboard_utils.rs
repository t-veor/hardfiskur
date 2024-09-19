//! Bitboard utility functions, primarily for generating lookup and magic
//! tables.

use num_derive::{FromPrimitive, ToPrimitive};

use crate::board::{Bitboard, Square};

/// Represents a possible ray direction on the board.
///
/// Used for generating and querying primitive bitboards that represent a ray
/// from a starting square.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, FromPrimitive, ToPrimitive)]
pub enum Direction {
    East = 0,
    North = 1,
    NorthEast = 2,
    NorthWest = 3,
    West = 4,
    South = 5,
    SouthWest = 6,
    SouthEast = 7,
}

/// Returns all knight attacks from a bitboard of knights.
///
/// This should not be used in move generation and should only be used to
/// calculate lookup tables.
///
/// # Examples
///
/// ```
/// # use hardfiskur_core::{
/// #     board::{Bitboard, Square},
/// #     move_gen::bitboard_utils::knight_attacks
/// # };
/// let knight = Bitboard::from_square(Square::D4);
/// assert_eq!(
///     knight_attacks(knight),
///     "
///         . . . . . . . .
///         . . . . . . . .
///         . . # . # . . .
///         . # . . . # . .
///         . . . . . . . .
///         . # . . . # . .
///         . . # . # . . .
///         . . . . . . . .
///     ".parse().unwrap()
/// );
/// ```
pub fn knight_attacks(b: Bitboard) -> Bitboard {
    const NOT_A_FILE: Bitboard = Bitboard::A_FILE.not();
    const NOT_AB_FILE: Bitboard = Bitboard::A_FILE.or(Bitboard::B_FILE).not();
    const NOT_H_FILE: Bitboard = Bitboard::H_FILE.not();
    const NOT_GH_FILE: Bitboard = Bitboard::G_FILE.or(Bitboard::H_FILE).not();

    let mut attacks = Bitboard::EMPTY;

    attacks |= (b << 17) & NOT_A_FILE;
    attacks |= (b << 10) & NOT_AB_FILE;
    attacks |= (b >> 6) & NOT_AB_FILE;
    attacks |= (b >> 15) & NOT_A_FILE;
    attacks |= (b << 15) & NOT_H_FILE;
    attacks |= (b << 6) & NOT_GH_FILE;
    attacks |= (b >> 10) & NOT_GH_FILE;
    attacks |= (b >> 17) & NOT_H_FILE;

    attacks
}

/// Returns all king moves from a bitboard of kings.
///
/// This should not be used in move generation and should only be used to
/// calculate lookup tables.
///
/// # Examples
///
/// ```
/// # use hardfiskur_core::{
/// #     board::{Bitboard, Square},
/// #     move_gen::bitboard_utils::king_moves
/// # };
/// let king = Bitboard::from_square(Square::D4);
/// assert_eq!(
///     king_moves(king),
///     "
///         . . . . . . . .
///         . . . . . . . .
///         . . . . . . . .
///         . . # # # . . .
///         . . # . # . . .
///         . . # # # . . .
///         . . . . . . . .
///         . . . . . . . .
///     ".parse().unwrap()
/// );
/// ```
pub fn king_moves(b: Bitboard) -> Bitboard {
    let mut attacks = b.step_east() | b.step_west();
    let tmp = b | attacks;
    attacks |= tmp.step_north() | tmp.step_south();

    attacks
}

pub(super) fn unblocked_ray_attacks(b: Bitboard, dir: Direction) -> Bitboard {
    let step_fn = match dir {
        Direction::East => Bitboard::step_east,
        Direction::North => Bitboard::step_north,
        Direction::NorthEast => Bitboard::step_north_east,
        Direction::NorthWest => Bitboard::step_north_west,
        Direction::West => Bitboard::step_west,
        Direction::South => Bitboard::step_south,
        Direction::SouthWest => Bitboard::step_south_west,
        Direction::SouthEast => Bitboard::step_south_east,
    };

    let mut attacks = step_fn(b);
    loop {
        let new_attacks = attacks | step_fn(attacks);
        if new_attacks == attacks {
            break;
        }
        attacks = new_attacks;
    }

    attacks
}

fn positive_ray_attacks(
    occupied: Bitboard,
    square: Square,
    dir: Direction,
    ray_attacks: &[[Bitboard; 8]; 64],
) -> Bitboard {
    let attacks = ray_attacks[square.index()][dir as usize];
    let blocker = attacks & occupied;
    let block_square = (blocker | Bitboard(0x8000000000000000)).lsb().unwrap();
    attacks ^ ray_attacks[block_square as usize][dir as usize]
}

fn negative_ray_attacks(
    occupied: Bitboard,
    square: Square,
    dir: Direction,
    ray_attacks: &[[Bitboard; 8]; 64],
) -> Bitboard {
    let attacks = ray_attacks[square.index()][dir as usize];
    let blocker = attacks & occupied;
    let block_square = (blocker | Bitboard(1)).msb().unwrap();
    attacks ^ ray_attacks[block_square as usize][dir as usize]
}

fn diagonal_attacks(
    occupied: Bitboard,
    square: Square,
    ray_attacks: &[[Bitboard; 8]; 64],
) -> Bitboard {
    positive_ray_attacks(occupied, square, Direction::NorthEast, ray_attacks)
        | negative_ray_attacks(occupied, square, Direction::SouthWest, ray_attacks)
}

fn antidiagonal_attacks(
    occupied: Bitboard,
    square: Square,
    ray_attacks: &[[Bitboard; 8]; 64],
) -> Bitboard {
    positive_ray_attacks(occupied, square, Direction::NorthWest, ray_attacks)
        | negative_ray_attacks(occupied, square, Direction::SouthEast, ray_attacks)
}

fn file_attacks(occupied: Bitboard, square: Square, ray_attacks: &[[Bitboard; 8]; 64]) -> Bitboard {
    positive_ray_attacks(occupied, square, Direction::North, ray_attacks)
        | negative_ray_attacks(occupied, square, Direction::South, ray_attacks)
}

fn rank_attacks(occupied: Bitboard, square: Square, ray_attacks: &[[Bitboard; 8]; 64]) -> Bitboard {
    positive_ray_attacks(occupied, square, Direction::East, ray_attacks)
        | negative_ray_attacks(occupied, square, Direction::West, ray_attacks)
}

/// Returns rook attacks from the starting square.
///
/// Note that if the attack is blocked by bits in the `occupied` bitboard, the
/// attacks will include the square the blocker is encountered (but none
/// after the blocker).
///
/// This should not be used in move generation and should only be used to
/// calculate lookup tables.
///
/// `ray_attacks` should be a valid pre-populated look up table, which can be
/// obtained from [`crate::move_gen::lookups::gen_ray_attacks`].
///
/// # Examples
///
/// ```
/// # use hardfiskur_core::{
/// #     board::{Bitboard, Square},
/// #     move_gen::{lookups::gen_ray_attacks, bitboard_utils::rook_attacks}
/// # };
/// let ray_attacks = gen_ray_attacks();
/// let occupied = "
///         . . . . . . . .
///         . . . . # . . .
///         . . . . . . . .
///         . . . . # . . .
///         . . # . . . . .
///         . . . . . . . .
///         . . . . . . . .
///         . . . . . . . .
/// ".parse().unwrap();
/// assert_eq!(
///     rook_attacks(occupied, Square::E4, &ray_attacks),
///     "
///         . . . . . . . .
///         . . . . . . . .
///         . . . . . . . .
///         . . . . # . . .
///         . . # # . # # #
///         . . . . # . . .
///         . . . . # . . .
///         . . . . # . . .
///     ".parse().unwrap(),
/// );
/// ```
pub fn rook_attacks(
    occupied: Bitboard,
    square: Square,
    ray_attacks: &[[Bitboard; 8]; 64],
) -> Bitboard {
    file_attacks(occupied, square, ray_attacks) | rank_attacks(occupied, square, ray_attacks)
}

/// Returns bishop attacks from the starting square.
///
/// Note that if the attack is blocked by bits in the `occupied` bitboard, the
/// attacks will include the square the blocker is encountered (but none
/// after the blocker).
///
/// This should not be used in move generation and should only be used to
/// calculate lookup tables.
///
/// `ray_attacks` should be a valid pre-populated look up table, which can be
/// obtained from [`crate::move_gen::lookups::gen_ray_attacks`].
///
/// # Examples
///
/// ```
/// # use hardfiskur_core::{
/// #     board::{Bitboard, Square},
/// #     move_gen::{lookups::gen_ray_attacks, bitboard_utils::bishop_attacks}
/// # };
/// let ray_attacks = gen_ray_attacks();
/// let occupied = "
///         . . . . . . . .
///         . # . . . . . .
///         . . . . . . . .
///         . . . # . . . .
///         . . . . . . . .
///         . . . . . . . .
///         . . # . . . . .
///         . . . . . . . .
/// ".parse().unwrap();
/// assert_eq!(
///     bishop_attacks(occupied, Square::E4, &ray_attacks),
///     "
///         . . . . . . . .
///         . . . . . . . #
///         . . . . . . # .
///         . . . # . # . .
///         . . . . . . . .
///         . . . # . # . .
///         . . # . . . # .
///         . . . . . . . #
///     ".parse().unwrap(),
/// );
/// ```
pub fn bishop_attacks(
    occupied: Bitboard,
    square: Square,
    ray_attacks: &[[Bitboard; 8]; 64],
) -> Bitboard {
    diagonal_attacks(occupied, square, ray_attacks)
        | antidiagonal_attacks(occupied, square, ray_attacks)
}

/// Returns queen attacks from the starting square.
///
/// This is simply the union of [`bishop_attacks`] and [`rook_attacks`].
///
/// Note that if the attack is blocked by bits in the `occupied` bitboard, the
/// attacks will include the square the blocker is encountered (but none
/// after the blocker).
///
/// This should not be used in move generation and should only be used to
/// calculate lookup tables.
///
/// `ray_attacks` should be a valid pre-populated look up table, which can be
/// obtained from [`crate::move_gen::lookups::gen_ray_attacks`].
pub fn queen_attacks(
    occupied: Bitboard,
    square: Square,
    ray_attacks: &[[Bitboard; 8]; 64],
) -> Bitboard {
    rook_attacks(occupied, square, ray_attacks) | bishop_attacks(occupied, square, ray_attacks)
}

/// For a rook on the given square, returns a bitboard of the squares from which
/// pieces may block the rook.
///
/// Note since that attacks always include the first blocker in a direction, the
/// last square in each direction does not affect attacks and is left out of
/// this mask.
///
/// `ray_attacks` should be a valid pre-populated look up table, which can be
/// obtained from [`crate::move_gen::lookups::gen_ray_attacks`].
///
/// # Examples
///
/// ```
/// # use hardfiskur_core::{
/// #     board::Square,
/// #     move_gen::{lookups::gen_ray_attacks, bitboard_utils::rook_attack_blocker_mask}
/// # };
/// let ray_attacks = gen_ray_attacks();
/// assert_eq!(
///     rook_attack_blocker_mask(Square::D3, &ray_attacks),
///     "
///         . . . . . . . .
///         . . . # . . . .
///         . . . # . . . .
///         . . . # . . . .
///         . . . # . . . .
///         . # # . # # # .
///         . . . # . . . .
///         . . . . . . . .
///     "
///     .parse()
///     .unwrap()
/// );
/// ```
pub fn rook_attack_blocker_mask(square: Square, ray_attacks: &[[Bitboard; 8]; 64]) -> Bitboard {
    let vertical_mask = (ray_attacks[square.index()][Direction::North as usize]
        | ray_attacks[square.index()][Direction::South as usize])
        .without(Bitboard::RANK_1 | Bitboard::RANK_8);
    let horizontal_mask = (ray_attacks[square.index()][Direction::East as usize]
        | ray_attacks[square.index()][Direction::West as usize])
        .without(Bitboard::A_FILE | Bitboard::H_FILE);

    vertical_mask | horizontal_mask
}

/// For a bishop on the given square, returns a bitboard of the squares from
/// which pieces may block the bishop.
///
/// Note since that attacks always include the first blocker in a direction, the
/// last square in each direction does not affect attacks and is left out of
/// this mask.
///
/// `ray_attacks` should be a valid pre-populated look up table, which can be
/// obtained from [`crate::move_gen::lookups::gen_ray_attacks`].
///
/// # Examples
///
/// ```
/// # use hardfiskur_core::{
/// #     board::Square,
/// #     move_gen::{lookups::gen_ray_attacks, bitboard_utils::bishop_attack_blocker_mask}
/// # };
/// let ray_attacks = gen_ray_attacks();
/// assert_eq!(
///     bishop_attack_blocker_mask(Square::D3, &ray_attacks),
///     "
///         . . . . . . . .
///         . . . . . . . .
///         . . . . . . # .
///         . # . . . # . .
///         . . # . # . . .
///         . . . . . . . .
///         . . # . # . . .
///         . . . . . . . .
///     "
///     .parse()
///     .unwrap()
/// );
/// ```
pub fn bishop_attack_blocker_mask(square: Square, ray_attacks: &[[Bitboard; 8]; 64]) -> Bitboard {
    let board_edge = Bitboard::RANK_1 | Bitboard::RANK_8 | Bitboard::A_FILE | Bitboard::H_FILE;

    (ray_attacks[square.index()][Direction::NorthEast as usize]
        | ray_attacks[square.index()][Direction::NorthWest as usize]
        | ray_attacks[square.index()][Direction::SouthWest as usize]
        | ray_attacks[square.index()][Direction::SouthEast as usize])
        .without(board_edge)
}

/// Transfers the low bits of `n` according to the provided mask.
///
/// This is sometimes known as the Parallel Bits Deposit or Scatter Bits
/// operation.
///
/// When `mask` is is a attack blocker mask as generated by
/// [`rook_attack_blocker_mask`] or [`bishop_attack_blocker_mask`], iterating
/// through `n` in the range `0..1 << mask.pop_count()` will iterate through
/// all possible blocker arrangements for that mask.
///
/// # Examples
///
/// ```
/// # use hardfiskur_core::{
/// #     board::Bitboard,
/// #     move_gen::bitboard_utils::nth_blocker_arrangement_for_mask,
/// # };
/// let mask     = 0b11001011;
/// let n        = 0b00010101;
/// let expected = 0b10001001;
/// //               ^^  ^ ^^ notice how the bits of n have been distributed here
/// assert_eq!(nth_blocker_arrangement_for_mask(n, Bitboard(mask)), Bitboard(expected));
/// ```
pub fn nth_blocker_arrangement_for_mask(mut n: usize, mask: Bitboard) -> Bitboard {
    let mut result = 0u64;
    for i in mask.bits() {
        result |= ((n & 1) as u64) << i;
        n >>= 1;
    }
    Bitboard(result)
}

#[cfg(test)]
mod test {
    use crate::move_gen::lookups::gen_ray_attacks;
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_knight_attacks() {
        assert_eq!(
            knight_attacks(Bitboard::from_square(Square::D4)),
            "
                . . . . . . . .
                . . . . . . . .
                . . # . # . . .
                . # . . . # . .
                . . . . . . . .
                . # . . . # . .
                . . # . # . . .
                . . . . . . . .
            "
            .parse()
            .unwrap()
        );

        assert_eq!(
            knight_attacks(Bitboard::from_square(Square::D4) | Bitboard::from_square(Square::E5)),
            "
                . . . . . . . .
                . . . # . # . .
                . . # . # . # .
                . # . . . # . .
                . . # . . . # .
                . # . # . # . .
                . . # . # . . .
                . . . . . . . .
            "
            .parse()
            .unwrap()
        );

        assert_eq!(
            knight_attacks(Bitboard::from_square(Square::B2)),
            "
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                # . # . . . . .
                . . . # . . . .
                . . . . . . . .
                . . . # . . . .
            "
            .parse()
            .unwrap()
        );

        assert_eq!(
            knight_attacks(Bitboard::from_square(Square::A1)),
            "
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . # . . . . . .
                . . # . . . . .
                . . . . . . . .
            "
            .parse()
            .unwrap()
        );

        assert_eq!(
            knight_attacks(Bitboard::from_square(Square::G7)),
            "
                . . . . # . . .
                . . . . . . . .
                . . . . # . . .
                . . . . . # . #
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
            "
            .parse()
            .unwrap()
        );

        assert_eq!(
            knight_attacks(Bitboard::from_square(Square::H8)),
            "
                . . . . . . . .
                . . . . . # . .
                . . . . . . # .
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
    fn test_king_moves() {
        assert_eq!(
            king_moves(Bitboard::from_square(Square::D4)),
            "
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . # # # . . .
                . . # . # . . .
                . . # # # . . .
                . . . . . . . .
                . . . . . . . .
            "
            .parse()
            .unwrap()
        );

        assert_eq!(
            king_moves(Bitboard::from_square(Square::E1)),
            "
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . # # # . .
                . . . # . # . .
            "
            .parse()
            .unwrap()
        );
    }

    #[test]
    fn test_rook_attacks() {
        let ray_attacks = gen_ray_attacks();
        assert_eq!(
            rook_attacks(Bitboard::EMPTY, Square::E4, &ray_attacks),
            "
                . . . . # . . .
                . . . . # . . .
                . . . . # . . .
                . . . . # . . .
                # # # # . # # #
                . . . . # . . .
                . . . . # . . .
                . . . . # . . .
            "
            .parse()
            .unwrap(),
        );

        let occupied = "
                . . . . . . . .
                . . . . # . . .
                . . . . . . . .
                . . . . # . . .
                . . # . # . . .
                . . . . . . . .
                . . . # . . . .
                . . . . # . . .
        "
        .parse()
        .unwrap();
        assert_eq!(
            rook_attacks(occupied, Square::E4, &ray_attacks),
            "
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . # . . .
                . . # # . # # #
                . . . . # . . .
                . . . . # . . .
                . . . . # . . .
            "
            .parse()
            .unwrap(),
        );
    }

    #[test]
    fn test_bishop_attacks() {
        let ray_attacks = gen_ray_attacks();
        assert_eq!(
            bishop_attacks(Bitboard::EMPTY, Square::E4, &ray_attacks),
            "
                # . . . . . . .
                . # . . . . . #
                . . # . . . # .
                . . . # . # . .
                . . . . . . . .
                . . . # . # . .
                . . # . . . # .
                . # . . . . . #
            "
            .parse()
            .unwrap(),
        );

        let occupied = "
                . . . . . . . .
                . # . . . . . .
                . . . . . . . .
                . . . # . . . .
                . . . # # . . .
                . . . . . . . .
                . . # . . . . .
                . . . . . . . .
        "
        .parse()
        .unwrap();
        assert_eq!(
            bishop_attacks(occupied, Square::E4, &ray_attacks),
            "
                . . . . . . . .
                . . . . . . . #
                . . . . . . # .
                . . . # . # . .
                . . . . . . . .
                . . . # . # . .
                . . # . . . # .
                . . . . . . . #
            "
            .parse()
            .unwrap(),
        );
    }

    #[test]
    fn test_queen_attacks() {
        let ray_attacks = gen_ray_attacks();
        assert_eq!(
            queen_attacks(Bitboard::EMPTY, Square::E4, &ray_attacks),
            "
                # . . . # . . .
                . # . . # . . #
                . . # . # . # .
                . . . # # # . .
                # # # # . # # #
                . . . # # # . .
                . . # . # . # .
                . # . . # . . #
            "
            .parse()
            .unwrap(),
        );

        let occupied = "
                . . . . . . . .
                . . . . . . . .
                . . . . # . . .
                . . . # . . . .
                . . . . # # # .
                . . . # . . . .
                . . # . # . . .
                . . . . . . . .
        "
        .parse()
        .unwrap();
        assert_eq!(
            queen_attacks(occupied, Square::E4, &ray_attacks),
            "
                . . . . . . . .
                . . . . . . . #
                . . . . # . # .
                . . . # # # . .
                # # # # . # . .
                . . . # # # . .
                . . . . # . # .
                . . . . . . . #
            "
            .parse()
            .unwrap(),
        );
    }

    #[test]
    fn test_rook_attack_blocker_mask() {
        let ray_attacks = gen_ray_attacks();

        assert_eq!(
            rook_attack_blocker_mask(Square::A1, &ray_attacks),
            "
                . . . . . . . .
                # . . . . . . .
                # . . . . . . .
                # . . . . . . .
                # . . . . . . .
                # . . . . . . .
                # . . . . . . .
                . # # # # # # .
            "
            .parse()
            .unwrap()
        );

        assert_eq!(
            rook_attack_blocker_mask(Square::D3, &ray_attacks),
            "
                . . . . . . . .
                . . . # . . . .
                . . . # . . . .
                . . . # . . . .
                . . . # . . . .
                . # # . # # # .
                . . . # . . . .
                . . . . . . . .
            "
            .parse()
            .unwrap()
        );
    }

    #[test]
    fn test_bishop_attack_blocker_mask() {
        let ray_attacks = gen_ray_attacks();

        assert_eq!(
            bishop_attack_blocker_mask(Square::A1, &ray_attacks),
            "
                . . . . . . . .
                . . . . . . # .
                . . . . . # . .
                . . . . # . . .
                . . . # . . . .
                . . # . . . . .
                . # . . . . . .
                . . . . . . . .
            "
            .parse()
            .unwrap()
        );

        assert_eq!(
            bishop_attack_blocker_mask(Square::D3, &ray_attacks),
            "
                . . . . . . . .
                . . . . . . . .
                . . . . . . # .
                . # . . . # . .
                . . # . # . . .
                . . . . . . . .
                . . # . # . . .
                . . . . . . . .
            "
            .parse()
            .unwrap()
        );
    }

    #[test]
    fn test_nth_blocker_arrangement_for_mask() {
        let mask = 0b11001010;
        let cases = [
            (0, 0b00000000),
            (1, 0b00000010),
            (2, 0b00001000),
            (3, 0b00001010),
            (4, 0b01000000),
            (5, 0b01000010),
            (6, 0b01001000),
            (7, 0b01001010),
            (8, 0b10000000),
            (9, 0b10000010),
            (10, 0b10001000),
            (11, 0b10001010),
            (12, 0b11000000),
            (13, 0b11000010),
            (14, 0b11001000),
            (15, 0b11001010),
        ];

        for (n, expected) in cases {
            assert_eq!(
                nth_blocker_arrangement_for_mask(n, Bitboard(mask)),
                Bitboard(expected)
            );
        }
    }
}
