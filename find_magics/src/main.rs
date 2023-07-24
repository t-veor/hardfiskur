use rand::{thread_rng, Rng};

use hardfiskur_core::{
    board::{Bitboard, Square},
    move_gen::{
        bitboard_utils::{
            bishop_attack_blocker_mask, bishop_attacks, nth_blocker_arrangement_for_mask,
            rook_attack_blocker_mask, rook_attacks,
        },
        lookups::gen_ray_attacks,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MagicType {
    Bishop,
    Rook,
}

fn random_u64_few_bits(r: &mut (impl Rng + ?Sized)) -> u64 {
    r.next_u64() & r.next_u64() & r.next_u64()
}

fn get_magic_index(board: Bitboard, magic: u64, num_bits: u32) -> usize {
    ((board.0.wrapping_mul(magic)) >> (64 - num_bits)) as usize
}

fn is_magic(
    all_blocker_arrangements: &[Bitboard],
    attacks_for_blocker_arrangements: &[Bitboard],
    num_bits: u32,
    magic: u64,
) -> bool {
    assert!(all_blocker_arrangements.len() == attacks_for_blocker_arrangements.len());

    let mut used_table = vec![Bitboard::EMPTY; 1 << num_bits];

    for (&blocker_arrangement, &attacks) in all_blocker_arrangements
        .iter()
        .zip(attacks_for_blocker_arrangements)
    {
        let magic_index = get_magic_index(blocker_arrangement, magic, num_bits);
        if used_table[magic_index] == Bitboard::EMPTY {
            used_table[magic_index] = attacks;
        } else if used_table[magic_index] != attacks {
            return false;
        }
    }

    true
}

fn find_magic(
    square: Square,
    magic_type: MagicType,
    target_bits: u32,
    search_limit: usize,
    r: &mut (impl Rng + ?Sized),
    ray_attacks: &[[Bitboard; 8]; 64],
) -> Option<u64> {
    let mask = match magic_type {
        MagicType::Bishop => bishop_attack_blocker_mask(square, ray_attacks),
        MagicType::Rook => rook_attack_blocker_mask(square, ray_attacks),
    };

    let num_bits_in_mask = mask.pop_count();
    let all_blocker_arrangements = (0..1 << num_bits_in_mask)
        .map(|n| nth_blocker_arrangement_for_mask(n, mask))
        .collect::<Vec<_>>();
    let attacks_for_blocker_arrangements = all_blocker_arrangements
        .iter()
        .map(|&blockers| match magic_type {
            MagicType::Bishop => bishop_attacks(blockers, square, ray_attacks),
            MagicType::Rook => rook_attacks(blockers, square, ray_attacks),
        })
        .collect::<Vec<_>>();

    for _ in 0..search_limit {
        let magic = random_u64_few_bits(r);
        if is_magic(
            &all_blocker_arrangements,
            &attacks_for_blocker_arrangements,
            target_bits,
            magic,
        ) {
            return Some(magic);
        }
    }

    None
}

#[rustfmt::skip]
const ROOK_TARGET_BITS: [u32; 64] = [
    12, 11, 11, 11, 11, 11, 11, 12,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    12, 11, 11, 11, 11, 11, 11, 12,
];

#[rustfmt::skip]
const BISHOP_TARGET_BITS: [u32; 64] = [
    6, 5, 5, 5, 5, 5, 5, 6,
    5, 5, 5, 5, 5, 5, 5, 5,
    5, 5, 7, 7, 7, 7, 5, 5,
    5, 5, 7, 9, 9, 7, 5, 5,
    5, 5, 7, 9, 9, 7, 5, 5,
    5, 5, 7, 7, 7, 7, 5, 5,
    5, 5, 5, 5, 5, 5, 5, 5,
    6, 5, 5, 5, 5, 5, 5, 6,
];

fn main() {
    let ray_attacks = gen_ray_attacks();
    let mut rng = thread_rng();

    println!("const ROOK_MAGICS: [(u64, u32); 64] = [");
    for (i, &target_bits) in ROOK_TARGET_BITS.iter().enumerate() {
        let square = Square::from_index_unchecked(i);
        let magic = find_magic(
            square,
            MagicType::Rook,
            target_bits,
            1_000_000,
            &mut rng,
            &ray_attacks,
        )
        .expect("Could not find rook magic :(");
        println!("    (0x{magic:016x}, {target_bits}),");
    }
    println!("];");

    println!();

    println!("const BISHOP_MAGICS: [(u64, u32); 64] = [");
    for (i, &target_bits) in BISHOP_TARGET_BITS.iter().enumerate() {
        let square = Square::from_index_unchecked(i);
        let magic = find_magic(
            square,
            MagicType::Bishop,
            target_bits,
            1_000_000,
            &mut rng,
            &ray_attacks,
        )
        .expect("Could not find bishop magic :(");
        println!("    (0x{magic:016x}, {target_bits}),");
    }
    println!("];");
}
