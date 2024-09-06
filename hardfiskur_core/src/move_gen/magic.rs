//! Magic bitboards implementation.
//!
//! Magic bitboards are a fast perfect hashing algorithm intended to very
//! quickly retrieve the allowed attack pattern for sliding pieces, taking into
//! account any blocking pieces along the way.
//!
//! A naive approach to calculating the attack pattern for a sliding piece may
//! be to loop moving away from the piece in a direction until another blocking
//! piece is found, and return the squares which were iterated over as the
//! allowed attack pattern for this piece. This is however very slow.
//!
//! The classical approach in contrast attempts to speed this up using a
//! pre-computed table of rays in every direction from every starting square.
//! Calculating the allowed attacks in any direction amounts to two lookups and
//! a bitscan:
//! 1. Get the ray attack in the desired direction from the starting square.
//! 2. Mask off the occupied set with the ray attack.
//! 3. Perform a bitscan to find the first blocker the ray will encounter in the
//!    masked occupied set.
//! 4. Get the ray starting from the first blocker in the same direction, and
//!    remove it from the initial ray attack.
//!
//! This will return truncated ray stopping after the first blocker. Note that
//! the direction of the bitscan will depend on the direction of the ray -- for
//! "positive" directions where the square indices increase as you move along a
//! ray, a forward bitscan is needed, and a reverse bitscan is needed for
//! "negative" directions.
//!
//! Magic bitboards however can do this with only one table lookup and no need
//! for bitscanning. The idea is this: suppose a rook is on E4. Its attack
//! pattern will be affected by potential blocking pieces, but for this rook
//! only blocking pieces on the E file and the 4th rank will matter. Since
//! attack patterns include the first blocker in any direction, blockers on the
//! last square in any direction in fact do not affect the attack pattern at
//! all. This means that the squares that may affect the attack pattern of our
//! E4 rook (which we will call the blocker mask) looks like this:
//!
//! ```txt
//! . . . . . . . .
//! . . . . # . . .
//! . . . . # . . .
//! . . . . # . . .
//! . # # # . # # .
//! . . . . # . . .
//! . . . . # . . .
//! . . . . . . . .
//! ```
//!
//! This means given a blocker bitboard, there are only 10 bits that actually
//! matter for the attack pattern of our rook (on E4. Other squares have
//! different numbers of bits; e.g. A1 has 12 bits in its blocker mask). We
//! could collect these 10 bits off the blocker bitboard and flatten them into a
//! 10-bit number, and use them in a size-1024 lookup table which contains
//! pre-computed attack patterns for each of these blocker arrangements.
//! Attempting to gather these bits would require more slow iteration however,
//! but here's where the magic part comes in.
//!
//! For every square we will have a "magic number", which has the following
//! special property: multiplying any blocker arrangement for this square will
//! result in a number that has the blocker bits in some order, consecutively,
//! in the most significant bits. It will also have some garbage bits in the
//! less sigificant bits, but we don't care about those -- since the blocker
//! bits were effectively moved to the front and packed together, we'll simply
//! right shift the product to discard the garbage bits and extract resulting
//! blocker bits. We can then immediately use this as an index in a lookup
//! table.
//!
//! So, calculating the attack patterns for a rook/bishop simply amounts to
//! this:
//! 1. Mask the occupied set by the blocker mask for this square.
//! 2. Multiply the resulting bitboard by the magic number for this square.
//! 3. Right shift the product by some amount (depending on the magic number) to
//!    extract an index.
//! 4. Use the index to lookup the pre-computed attack pattern.
//!
//! This is much faster and requires no iteration or bitscanning.
//!
//! We don't actually know of a way of finding these "magic" numbers
//! mathematically, or even if they exist, but in practice we can simply check
//! random numbers until we stumble across ones that work.
//!
//! More details about magic bitboards can be found here:
//! <https://www.chessprogramming.org/Magic_Bitboards>

use std::{ops::Range, sync::OnceLock};

use crate::board::{Bitboard, Square};

use super::bitboard_utils::{
    bishop_attack_blocker_mask, bishop_attacks, nth_blocker_arrangement_for_mask,
    rook_attack_blocker_mask, rook_attacks,
};

// Rook magic numbers, found by the find_magics binary.
const ROOK_MAGICS: [(u64, u32); 64] = [
    (0x0480003020400580, 12),
    (0x8040042000100040, 11),
    (0x00800a6000821001, 11),
    (0x0080080080100204, 11),
    (0x0200042002001009, 11),
    (0x2100020400810008, 11),
    (0x4480010020800200, 11),
    (0x0080012851000080, 12),
    (0x0006800088624000, 11),
    (0x2110404000201000, 10),
    (0x1049002001003840, 10),
    (0x9c32004200202811, 10),
    (0x0021000800050150, 10),
    (0x0232000854060010, 10),
    (0x0004002c02015028, 10),
    (0x0008800041000580, 11),
    (0x000321800040018a, 11),
    (0x0004404010002002, 10),
    (0x0010808010042000, 10),
    (0x0412020008402030, 10),
    (0x0808008008ac0080, 10),
    (0x0001a80104102040, 10),
    (0x0000940010028805, 10),
    (0x0003220004804104, 11),
    (0x0083800480224000, 11),
    (0x1800200040100044, 10),
    (0x9201044100122000, 10),
    (0x0000100080800800, 10),
    (0x2008080180040080, 10),
    (0x1201860080040080, 10),
    (0x0100210c00100882, 10),
    (0x0020040600008841, 11),
    (0x08824000808000a0, 11),
    (0x5010004000402001, 10),
    (0x0410882000801000, 10),
    (0x0022210009001000, 10),
    (0x5218080101001084, 10),
    (0x0004244008012010, 10),
    (0x0100900984004812, 10),
    (0x0100008042000124, 11),
    (0x0000a08440008000, 11),
    (0x09042000500a4000, 10),
    (0x0000102200820040, 10),
    (0x8005002110010008, 10),
    (0x2008080100710004, 10),
    (0x200c000200048080, 10),
    (0x0c02010648240010, 10),
    (0x8040004c02820023, 11),
    (0x00010d204a018200, 11),
    (0x8110002001400440, 10),
    (0x0000620040841200, 10),
    (0x0908900248018080, 10),
    (0x4000810800040280, 10),
    (0x0051000214000900, 10),
    (0x4000100a01080400, 10),
    (0x8080114124008200, 11),
    (0x0080d50040228001, 12),
    (0x0422008210410022, 11),
    (0x00200151000aa041, 11),
    (0x0000100100240821, 11),
    (0x820200104448200e, 11),
    (0x007100080c004609, 11),
    (0x1442002401088806, 11),
    (0x0440040221124082, 12),
];

// Bishop magic numbers, found by the find_magics binary.
const BISHOP_MAGICS: [(u64, u32); 64] = [
    (0x184004c094010060, 6),
    (0x0822020849110002, 5),
    (0x1430041040420000, 5),
    (0x40821a0602020220, 5),
    (0x0194142020202402, 5),
    (0x00020210940000c0, 5),
    (0x002601100814009b, 5),
    (0x4201010090211880, 6),
    (0x00d1400c08020054, 5),
    (0x20000254014c0100, 5),
    (0x04a01002d200c00a, 5),
    (0x0010842401800046, 5),
    (0x2400020210204020, 5),
    (0x0074020802080400, 5),
    (0x000041029014a040, 5),
    (0x4302206104222001, 5),
    (0x1920288488300520, 5),
    (0x1004080805080600, 5),
    (0x4008012241810200, 7),
    (0x2402000420220040, 7),
    (0x1501008090400023, 7),
    (0x1005210202012008, 7),
    (0x1120800404c84802, 5),
    (0x09020080404a0801, 5),
    (0x0420080020422c08, 5),
    (0x48100420c8182080, 5),
    (0x8208080030404140, 7),
    (0x000c180083220040, 9),
    (0x0004040000410048, 9),
    (0x1200410002100200, 7),
    (0x4064008031180100, 5),
    (0x0281094004640402, 5),
    (0x0501241200413002, 5),
    (0x0410900800052800, 5),
    (0x08822490000804a0, 7),
    (0x0042200800290810, 9),
    (0x8001100400008060, 9),
    (0x0981080108020302, 7),
    (0x8108050240212800, 5),
    (0x1082004104020090, 5),
    (0x012218020a00c0a0, 5),
    (0x400401882c308800, 5),
    (0x0004442401081002, 7),
    (0x010c182011000800, 7),
    (0x404048010400c440, 7),
    (0x22202010c0400080, 7),
    (0x0102260816080100, 5),
    (0x0422084119020120, 5),
    (0x6900441004101010, 5),
    (0x0000220210054201, 5),
    (0x8400020094140000, 5),
    (0x0100042084040002, 5),
    (0x100000500a020040, 5),
    (0x0100202001024006, 5),
    (0x0010040908020080, 5),
    (0x5010020601460046, 5),
    (0x002480c242202004, 6),
    (0x6400008a8c100200, 5),
    (0x0000005282680829, 5),
    (0x0280204024840402, 5),
    (0x5000000028102405, 5),
    (0x8625082082020201, 5),
    (0x0032a04202020424, 5),
    (0x1090210344008202, 6),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MagicTableType {
    Bishop,
    Rook,
}

#[derive(Debug, Clone)]
struct MagicTableEntryIdx {
    attack_table: Range<usize>,
    mask: Bitboard,
    magic: u64,
    shift: u32,
}

impl MagicTableEntryIdx {
    fn initialize_table_buffer(
        magic: u64,
        num_bits: u32,
        square: Square,
        table_type: MagicTableType,
        ray_attacks: &[[Bitboard; 8]; 64],
        table_buffer: &mut Vec<Bitboard>,
    ) -> Self {
        let shift = 64 - num_bits;
        let table_size = 1usize << num_bits;

        let mask = match table_type {
            MagicTableType::Bishop => bishop_attack_blocker_mask(square, ray_attacks),
            MagicTableType::Rook => rook_attack_blocker_mask(square, ray_attacks),
        };

        let prev_end = table_buffer.len();
        table_buffer.extend(std::iter::repeat(Bitboard::EMPTY).take(table_size));
        let attack_table = &mut table_buffer[prev_end..];

        for i in 0..table_size {
            let blockers = nth_blocker_arrangement_for_mask(i, mask);
            let magic_index = blockers.0.wrapping_mul(magic) >> shift;
            attack_table[magic_index as usize] = match table_type {
                MagicTableType::Bishop => bishop_attacks(blockers, square, ray_attacks),
                MagicTableType::Rook => rook_attacks(blockers, square, ray_attacks),
            };
        }

        Self {
            attack_table: prev_end..prev_end + table_size,
            mask,
            magic,
            shift,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MagicTableEntry<'a> {
    pub attack_table: &'a [Bitboard],
    pub mask: Bitboard,
    pub magic: u64,
    pub shift: u32,
}

impl<'a> MagicTableEntry<'a> {
    fn new(idx: &MagicTableEntryIdx, attack_table: &'a [Bitboard]) -> Self {
        Self {
            attack_table: &attack_table[idx.attack_table.clone()],
            mask: idx.mask,
            magic: idx.magic,
            shift: idx.shift,
        }
    }

    fn get_attacks(&self, occupied: Bitboard) -> Bitboard {
        let occupied = occupied & self.mask;
        let magic_index = occupied.0.wrapping_mul(self.magic) >> self.shift;
        self.attack_table[magic_index as usize]
    }
}

fn calculate_attack_table_size() -> usize {
    BISHOP_MAGICS
        .iter()
        .map(|&(_magic, num_bits)| 1 << num_bits)
        .sum::<usize>()
        + ROOK_MAGICS
            .iter()
            .map(|&(_magic, num_bits)| 1 << num_bits)
            .sum::<usize>()
}

fn create_full_attack_table(
    ray_attacks: &[[Bitboard; 8]; 64],
    out_bishop_table: &mut Vec<MagicTableEntryIdx>,
    out_rook_table: &mut Vec<MagicTableEntryIdx>,
) -> Vec<Bitboard> {
    let mut attack_table = Vec::with_capacity(calculate_attack_table_size());

    *out_bishop_table = BISHOP_MAGICS
        .iter()
        .enumerate()
        .map(|(i, &(magic, num_bits))| {
            let square = Square::from_index_unchecked(i);
            MagicTableEntryIdx::initialize_table_buffer(
                magic,
                num_bits,
                square,
                MagicTableType::Bishop,
                ray_attacks,
                &mut attack_table,
            )
        })
        .collect();

    *out_rook_table = ROOK_MAGICS
        .iter()
        .enumerate()
        .map(|(i, &(magic, num_bits))| {
            let square = Square::from_index_unchecked(i);
            MagicTableEntryIdx::initialize_table_buffer(
                magic,
                num_bits,
                square,
                MagicTableType::Rook,
                ray_attacks,
                &mut attack_table,
            )
        })
        .collect();

    attack_table
}

static ATTACK_TABLE: OnceLock<Vec<Bitboard>> = OnceLock::new();
static MAGIC_TABLES: OnceLock<MagicTables> = OnceLock::new();

pub struct MagicTables {
    bishop_table: [MagicTableEntry<'static>; 64],
    rook_table: [MagicTableEntry<'static>; 64],
}

impl MagicTables {
    pub(super) fn get(ray_attacks: &[[Bitboard; 8]; 64]) -> &'static Self {
        MAGIC_TABLES.get_or_init(|| Self::new(ray_attacks))
    }

    // This function should only be called once!
    fn new(ray_attacks: &[[Bitboard; 8]; 64]) -> Self {
        let mut bishop_table = Vec::new();
        let mut rook_table = Vec::new();

        let attack_table = ATTACK_TABLE.get_or_init(|| {
            create_full_attack_table(ray_attacks, &mut bishop_table, &mut rook_table)
        });
        let bishop_table = bishop_table
            .into_iter()
            .map(|idx| MagicTableEntry::new(&idx, attack_table))
            .collect::<Vec<_>>();
        let rook_table = rook_table
            .into_iter()
            .map(|idx| MagicTableEntry::new(&idx, attack_table))
            .collect::<Vec<_>>();

        Self {
            bishop_table: bishop_table.try_into().unwrap(),
            rook_table: rook_table.try_into().unwrap(),
        }
    }

    pub fn bishop_attacks(&self, occupied: Bitboard, square: Square) -> Bitboard {
        self.bishop_table[square.index()].get_attacks(occupied)
    }

    pub fn rook_attacks(&self, occupied: Bitboard, square: Square) -> Bitboard {
        self.rook_table[square.index()].get_attacks(occupied)
    }

    pub fn debug_bishop_table(&self) -> &[MagicTableEntry<'static>; 64] {
        &self.bishop_table
    }

    pub fn debug_rook_table(&self) -> &[MagicTableEntry<'static>; 64] {
        &self.rook_table
    }
}

#[cfg(test)]
mod test {
    use crate::move_gen::lookups::gen_ray_attacks;

    use super::*;

    #[test]
    fn test_magic_bishop_tables() {
        let ray_attacks = gen_ray_attacks();
        let tables = MagicTables::get(&ray_attacks);

        for square in Square::all() {
            let mask = bishop_attack_blocker_mask(square, &ray_attacks);
            let num_bits = mask.pop_count();

            for n in 0..1 << num_bits {
                let occupied = nth_blocker_arrangement_for_mask(n, mask);

                assert_eq!(
                    tables.bishop_attacks(occupied, square),
                    bishop_attacks(occupied, square, &ray_attacks)
                );
            }
        }
    }

    #[test]
    fn test_magic_rook_tables() {
        let ray_attacks = gen_ray_attacks();
        let tables = MagicTables::get(&ray_attacks);

        for square in Square::all() {
            let mask = rook_attack_blocker_mask(square, &ray_attacks);
            let num_bits = mask.pop_count();

            for n in 0..1 << num_bits {
                let occupied = nth_blocker_arrangement_for_mask(n, mask);

                assert_eq!(
                    tables.rook_attacks(occupied, square),
                    rook_attacks(occupied, square, &ray_attacks)
                );
            }
        }
    }

    #[test]
    fn test_magic_masking() {
        let ray_attacks = gen_ray_attacks();
        let tables = MagicTables::get(&ray_attacks);

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
            tables.rook_attacks(occupied, Square::E4),
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
}
