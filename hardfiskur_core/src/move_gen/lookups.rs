use std::sync::OnceLock;

use num_traits::FromPrimitive;

use crate::board::{bitboard::Bitboard, Square};

use super::{
    bitboard_utils::{king_moves, knight_attacks, unblocked_ray_attacks, Direction},
    magic::MagicTables,
};

pub struct Lookups {
    knight_moves: [Bitboard; 64],
    king_moves: [Bitboard; 64],
    ray_attacks: [[Bitboard; 8]; 64],
    in_between: [[Bitboard; 64]; 64],

    magic: MagicTables,
}

static LOOKUPS: OnceLock<Lookups> = OnceLock::new();

impl Lookups {
    fn new() -> Self {
        let knight_moves = gen_knight_moves();
        let king_moves = gen_king_moves();
        let ray_attacks = gen_ray_attacks();
        let in_between = gen_in_between(&ray_attacks);

        let magic = MagicTables::new(&ray_attacks);

        Self {
            knight_moves,
            king_moves,
            ray_attacks,
            in_between,

            magic,
        }
    }

    pub fn get_instance() -> &'static Self {
        LOOKUPS.get_or_init(Self::new)
    }

    pub fn get_knight_moves(&self, square: Square) -> Bitboard {
        self.knight_moves[square.index()]
    }

    pub fn get_king_moves(&self, square: Square) -> Bitboard {
        self.king_moves[square.index()]
    }

    pub fn get_rook_attacks(&self, occupied: Bitboard, square: Square) -> Bitboard {
        self.magic.rook_attacks(occupied, square)
    }

    pub fn get_bishop_attacks(&self, occupied: Bitboard, square: Square) -> Bitboard {
        self.magic.bishop_attacks(occupied, square)
    }

    pub fn get_queen_attacks(&self, occupied: Bitboard, square: Square) -> Bitboard {
        self.get_rook_attacks(occupied, square) | self.get_bishop_attacks(occupied, square)
    }

    pub fn get_in_between(&self, from: Square, to: Square) -> Bitboard {
        self.in_between[from.index()][to.index()]
    }
}

pub fn gen_knight_moves() -> [Bitboard; 64] {
    let mut moves = [Bitboard::default(); 64];
    for (i, moves_from_square) in moves.iter_mut().enumerate() {
        *moves_from_square = knight_attacks(Bitboard::from_index(i as _));
    }
    moves
}

pub fn gen_king_moves() -> [Bitboard; 64] {
    let mut moves = [Bitboard::default(); 64];
    for (i, moves_from_square) in moves.iter_mut().enumerate() {
        *moves_from_square = king_moves(Bitboard::from_index(i as _));
    }
    moves
}

pub fn gen_ray_attacks() -> [[Bitboard; 8]; 64] {
    let mut attacks = [[Bitboard::default(); 8]; 64];

    for (i, attacks_from_square) in attacks.iter_mut().enumerate() {
        let base = Bitboard::from_index(i as _);

        for (dir, attacks_in_dir) in attacks_from_square.iter_mut().enumerate() {
            let dir_enum = Direction::from_usize(dir).unwrap();
            *attacks_in_dir = unblocked_ray_attacks(base, dir_enum)
        }
    }

    attacks
}

pub fn gen_in_between(ray_attacks: &[[Bitboard; 8]; 64]) -> [[Bitboard; 64]; 64] {
    let mut table = [[Bitboard::default(); 64]; 64];

    for from in 0..64 {
        for dir in 0..4 {
            let ray = ray_attacks[from][dir];
            for to in ray.bits() {
                let to = to as usize;
                let ray_between = ray ^ ray_attacks[to][dir] ^ Bitboard::from_index(to as _);
                table[from][to] = ray_between;
                table[to][from] = ray_between;
            }
        }
    }

    table
}
