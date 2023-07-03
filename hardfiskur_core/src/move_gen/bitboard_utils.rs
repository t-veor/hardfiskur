use num_derive::{FromPrimitive, ToPrimitive};

use crate::board::{bitboard::Bitboard, Square};

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromPrimitive, ToPrimitive)]
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

pub fn knight_attacks(b: Bitboard) -> Bitboard {
    // have to be written like this to avoid the fact that const traits are not stabilised...
    const NOT_A_FILE: Bitboard = Bitboard(!Bitboard::A_FILE.0);
    const NOT_AB_FILE: Bitboard = Bitboard(!Bitboard::A_FILE.0 & !Bitboard::B_FILE.0);
    const NOT_H_FILE: Bitboard = Bitboard(!Bitboard::H_FILE.0);
    const NOT_GH_FILE: Bitboard = Bitboard(!Bitboard::G_FILE.0 & !Bitboard::H_FILE.0);

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

pub fn king_moves(b: Bitboard) -> Bitboard {
    let mut attacks = b.step_east() | b.step_west();
    let tmp = b | attacks;
    attacks |= tmp.step_north() | tmp.step_south();

    attacks
}

pub fn unblocked_ray_attacks(b: Bitboard, dir: Direction) -> Bitboard {
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
    let block_square = (blocker | Bitboard(0x8000000000000000)).lsb();
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
    let block_square = (blocker | Bitboard(1)).msb();
    attacks ^ ray_attacks[block_square as usize][dir as usize]
}

pub fn diagonal_attacks(
    occupied: Bitboard,
    square: Square,
    ray_attacks: &[[Bitboard; 8]; 64],
) -> Bitboard {
    positive_ray_attacks(occupied, square, Direction::NorthEast, ray_attacks)
        | negative_ray_attacks(occupied, square, Direction::SouthWest, ray_attacks)
}

pub fn antidiagonal_attacks(
    occupied: Bitboard,
    square: Square,
    ray_attacks: &[[Bitboard; 8]; 64],
) -> Bitboard {
    positive_ray_attacks(occupied, square, Direction::NorthWest, ray_attacks)
        | negative_ray_attacks(occupied, square, Direction::SouthEast, ray_attacks)
}

pub fn file_attacks(
    occupied: Bitboard,
    square: Square,
    ray_attacks: &[[Bitboard; 8]; 64],
) -> Bitboard {
    positive_ray_attacks(occupied, square, Direction::North, ray_attacks)
        | negative_ray_attacks(occupied, square, Direction::South, ray_attacks)
}

pub fn rank_attacks(
    occupied: Bitboard,
    square: Square,
    ray_attacks: &[[Bitboard; 8]; 64],
) -> Bitboard {
    positive_ray_attacks(occupied, square, Direction::East, ray_attacks)
        | negative_ray_attacks(occupied, square, Direction::West, ray_attacks)
}

pub fn rook_attacks(
    occupied: Bitboard,
    square: Square,
    ray_attacks: &[[Bitboard; 8]; 64],
) -> Bitboard {
    file_attacks(occupied, square, ray_attacks) | rank_attacks(occupied, square, ray_attacks)
}

pub fn bishop_attacks(
    occupied: Bitboard,
    square: Square,
    ray_attacks: &[[Bitboard; 8]; 64],
) -> Bitboard {
    diagonal_attacks(occupied, square, ray_attacks)
        | antidiagonal_attacks(occupied, square, ray_attacks)
}

pub fn queen_attacks(
    occupied: Bitboard,
    square: Square,
    ray_attacks: &[[Bitboard; 8]; 64],
) -> Bitboard {
    rook_attacks(occupied, square, ray_attacks) | bishop_attacks(occupied, square, ray_attacks)
}

pub fn rook_attack_blocker_mask(square: Square, ray_attacks: &[[Bitboard; 8]; 64]) -> Bitboard {
    let vertical_mask = (ray_attacks[square.index()][Direction::North as usize]
        | ray_attacks[square.index()][Direction::South as usize])
        .without(Bitboard::RANK_1 | Bitboard::RANK_8);
    let horizontal_mask = (ray_attacks[square.index()][Direction::East as usize]
        | ray_attacks[square.index()][Direction::West as usize])
        .without(Bitboard::A_FILE | Bitboard::H_FILE);

    vertical_mask | horizontal_mask
}

pub fn bishop_attack_blocker_mask(square: Square, ray_attacks: &[[Bitboard; 8]; 64]) -> Bitboard {
    let board_edge = Bitboard::RANK_1 | Bitboard::RANK_8 | Bitboard::A_FILE | Bitboard::H_FILE;

    (ray_attacks[square.index()][Direction::NorthEast as usize]
        | ray_attacks[square.index()][Direction::NorthWest as usize]
        | ray_attacks[square.index()][Direction::SouthWest as usize]
        | ray_attacks[square.index()][Direction::SouthEast as usize])
        .without(board_edge)
}

pub fn nth_blocker_arrangement_for_mask(mut n: usize, mask: Bitboard) -> Bitboard {
    let mut result = 0u64;
    for i in mask.bits() {
        result |= ((n & 1) as u64) << i;
        n >>= 1;
    }
    Bitboard(result)
}
