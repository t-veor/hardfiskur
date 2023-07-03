use std::{ops::Range, sync::OnceLock};

use crate::board::{bitboard::Bitboard, Square};

use super::bitboard_utils::{
    bishop_attack_blocker_mask, bishop_attacks, nth_blocker_arrangement_for_mask,
    rook_attack_blocker_mask, rook_attacks,
};

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
struct MagicTableEntry<'a> {
    attack_table: &'a [Bitboard],
    mask: Bitboard,
    magic: u64,
    shift: u32,
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

struct StaticMagicTables {
    attack_table: Vec<Bitboard>,
    bishop_table_idxs: [MagicTableEntryIdx; 64],
    rook_table_idxs: [MagicTableEntryIdx; 64],
}

fn create_full_attack_table(ray_attacks: &[[Bitboard; 8]; 64]) -> StaticMagicTables {
    let mut attack_table = Vec::new();

    let bishop_table_idxs = BISHOP_MAGICS
        .iter()
        .enumerate()
        .map(|(i, &(magic, num_bits))| {
            let square = Square::from_index_unchecked(i);
            MagicTableEntryIdx::initialize_table_buffer(
                magic,
                num_bits,
                square,
                MagicTableType::Bishop,
                &ray_attacks,
                &mut attack_table,
            )
        })
        .collect::<Vec<_>>();

    let rook_table_idxs = ROOK_MAGICS
        .iter()
        .enumerate()
        .map(|(i, &(magic, num_bits))| {
            let square = Square::from_index_unchecked(i);
            MagicTableEntryIdx::initialize_table_buffer(
                magic,
                num_bits,
                square,
                MagicTableType::Rook,
                &ray_attacks,
                &mut attack_table,
            )
        })
        .collect::<Vec<_>>();

    StaticMagicTables {
        attack_table,
        bishop_table_idxs: bishop_table_idxs.try_into().unwrap(),
        rook_table_idxs: rook_table_idxs.try_into().unwrap(),
    }
}

static STATIC_MAGIC_TABLES: OnceLock<StaticMagicTables> = OnceLock::new();

pub struct MagicTables {
    bishop_table: [MagicTableEntry<'static>; 64],
    rook_table: [MagicTableEntry<'static>; 64],
}

impl MagicTables {
    pub fn new(ray_attacks: &[[Bitboard; 8]; 64]) -> Self {
        let static_tables =
            STATIC_MAGIC_TABLES.get_or_init(|| create_full_attack_table(ray_attacks));
        let bishop_table = static_tables
            .bishop_table_idxs
            .iter()
            .map(|idx| MagicTableEntry::new(idx, &static_tables.attack_table))
            .collect::<Vec<_>>();
        let rook_table = static_tables
            .rook_table_idxs
            .iter()
            .map(|idx| MagicTableEntry::new(idx, &static_tables.attack_table))
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
}
