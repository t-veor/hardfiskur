use std::{fmt::Display, num::NonZeroUsize, u32};

use hardfiskur_core::board::{Board, Move, OptionalMove, UCIMove, ZobristHash};
use zerocopy::FromZeroes;

use crate::score::Score;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TranspositionFlag {
    Exact,
    Lowerbound,
    Upperbound,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TranspositionEntry {
    pub flag: TranspositionFlag,
    pub depth: u32,
    score: Score,
    pub best_move: Option<Move>,
}

impl TranspositionEntry {
    pub fn new(
        flag: TranspositionFlag,
        depth: u32,
        score: Score,
        best_move: Option<Move>,
        ply_from_root: u32,
    ) -> Self {
        Self {
            flag,
            depth,
            score: score.sub_plies_for_mate(ply_from_root),
            best_move,
        }
    }

    pub fn get_score(&self, ply_from_root: u32) -> Score {
        self.score.add_plies_for_mate(ply_from_root)
    }
}

impl Display for TranspositionEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "depth={}", self.depth)?;
        writeln!(f, "score={} {:?}", self.score, self.flag)?;

        match self.best_move {
            Some(m) => write!(f, "best_move={}", UCIMove::from(m)),
            None => write!(f, "best_move=<none>"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, FromZeroes)]
enum TranspositionFlagInternal {
    #[default]
    None,
    Exact,
    Lowerbound,
    Upperbound,
}

impl From<TranspositionFlag> for TranspositionFlagInternal {
    fn from(value: TranspositionFlag) -> Self {
        match value {
            TranspositionFlag::Exact => Self::Exact,
            TranspositionFlag::Lowerbound => Self::Lowerbound,
            TranspositionFlag::Upperbound => Self::Upperbound,
        }
    }
}

impl TryFrom<TranspositionFlagInternal> for TranspositionFlag {
    type Error = ();

    fn try_from(value: TranspositionFlagInternal) -> Result<Self, Self::Error> {
        match value {
            TranspositionFlagInternal::None => Err(()),
            TranspositionFlagInternal::Exact => Ok(Self::Exact),
            TranspositionFlagInternal::Lowerbound => Ok(Self::Lowerbound),
            TranspositionFlagInternal::Upperbound => Ok(Self::Upperbound),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, FromZeroes)]
struct TranspositionEntryInternal {
    key: ZobristHash,
    flag: TranspositionFlagInternal,
    depth: u32,
    score: Score,
    best_move: OptionalMove,
}

pub struct TranspositionTable {
    num_entries: usize,
    hash_mask: usize,
    entries: Vec<TranspositionEntryInternal>,

    occupied: u64,
}

impl TranspositionTable {
    pub fn new(max_size_in_mb: NonZeroUsize) -> Self {
        let num_entries = Self::get_num_entries(max_size_in_mb);

        Self {
            num_entries,
            hash_mask: num_entries - 1,
            entries: vec![FromZeroes::new_zeroed(); num_entries],
            occupied: 0,
        }
    }

    fn get_num_entries(max_size_in_mb: NonZeroUsize) -> usize {
        let max_size_in_mb = max_size_in_mb.get();
        const BYTES_PER_MB: usize = 1024 * 1024;

        let entry_size = size_of::<TranspositionEntryInternal>();
        let max_entries = max_size_in_mb * BYTES_PER_MB / entry_size;

        let num_entries = 1 << (usize::BITS - max_entries.leading_zeros() - 1);

        debug_assert!(
            size_of::<TranspositionEntryInternal>() * num_entries <= max_size_in_mb * BYTES_PER_MB
        );

        num_entries
    }

    fn index(&self, key: ZobristHash) -> usize {
        key.0 as usize & self.hash_mask
    }

    pub fn get(&self, key: ZobristHash) -> Option<TranspositionEntry> {
        let index = self.index(key);

        let entry = self.entries[index];
        if entry.key != key {
            return None;
        }

        Some(TranspositionEntry {
            flag: entry.flag.try_into().ok()?,
            depth: entry.depth,
            score: entry.score,
            best_move: entry.best_move.as_option_move(),
        })
    }

    pub fn set(&mut self, key: ZobristHash, entry: TranspositionEntry) {
        let index = self.index(key);

        let entry = TranspositionEntryInternal {
            key,
            flag: entry.flag.into(),
            depth: entry.depth,
            score: entry.score,
            best_move: entry.best_move.into(),
        };

        // Always-replace
        if self.entries[index].flag == TranspositionFlagInternal::None {
            self.occupied += 1;
        }
        self.entries[index] = entry;
    }

    pub fn resize(&mut self, max_size_in_mb: NonZeroUsize) {
        self.num_entries = Self::get_num_entries(max_size_in_mb);
        self.hash_mask = self.num_entries - 1;
        self.clear();
    }

    pub fn clear(&mut self) {
        self.entries = vec![FromZeroes::new_zeroed(); self.num_entries];
        self.occupied = 0;
    }

    pub fn occupancy(&self) -> u64 {
        self.occupied * 1000 / self.entries.len() as u64
    }

    pub fn extract_pv(&self, board: &mut Board) -> Vec<Move> {
        let mut moves = Vec::new();
        let mut seen_hashes = Vec::new();

        let mut limit = 50;

        while let Some(entry) = self.get(board.zobrist_hash()) {
            seen_hashes.push(board.zobrist_hash());

            if let Some(m) = entry.best_move {
                board.push_move_unchecked(m);
                moves.push(m)
            } else {
                break;
            }

            if seen_hashes.contains(&board.zobrist_hash()) {
                break;
            }

            limit -= 1;
            if limit <= 0 {
                eprintln!("Reached PV limit, detected loop!");
                break;
            }
        }

        // Unwind the moves
        for _ in 0..moves.len() {
            board.pop_move();
        }

        moves
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use hardfiskur_core::board::{MoveBuilder, Piece, Square};
    use pretty_assertions::assert_eq;

    const BYTES_IN_MB: usize = 1024 * 1024;

    #[test]
    fn gets_correct_number_of_entries() {
        for case_mb in [
            1, 2, 3, 4, 6, 8, 12, 24, 25, 32, 64, 96, 128, 144, 200, 256, 512, 999, 1024,
        ] {
            let num_entries =
                TranspositionTable::get_num_entries(NonZeroUsize::new(case_mb).unwrap());

            let target_bytes = case_mb * BYTES_IN_MB;
            let minimum_bytes = target_bytes / 2;

            let used_bytes = num_entries * size_of::<TranspositionEntryInternal>();

            assert!(num_entries.is_power_of_two());
            assert!(used_bytes > minimum_bytes);
            assert!(used_bytes <= target_bytes);
        }
    }

    #[test]
    fn constructs_table_correctly() {
        let max_size_in_mb = NonZeroUsize::new(12).unwrap();

        let expected_entries = TranspositionTable::get_num_entries(max_size_in_mb);
        let tt = TranspositionTable::new(max_size_in_mb);

        assert_eq!(tt.entries.len(), expected_entries);
        assert_eq!(tt.num_entries, expected_entries);
        assert_eq!(tt.hash_mask, expected_entries - 1);
        assert_eq!(tt.occupancy(), 0);
    }

    #[test]
    fn resizes_table_correctly() {
        let mut tt = TranspositionTable::new(8.try_into().unwrap());

        let max_size_in_mb = NonZeroUsize::new(2).unwrap();
        let expected_entries = TranspositionTable::get_num_entries(max_size_in_mb);

        tt.resize(max_size_in_mb);

        assert_eq!(tt.entries.len(), expected_entries);
        assert_eq!(tt.num_entries, expected_entries);
        assert_eq!(tt.hash_mask, expected_entries - 1);
        assert_eq!(tt.occupancy(), 0);
    }

    #[test]
    fn set_and_get() {
        let mut tt = TranspositionTable::new(1.try_into().unwrap());

        let entry = TranspositionEntry {
            flag: TranspositionFlag::Lowerbound,
            depth: 2,
            score: Score(1234),
            best_move: Some(MoveBuilder::new(Square::E2, Square::E4, Piece::WHITE_PAWN).build()),
        };

        tt.set(ZobristHash(0), entry.clone());

        assert_eq!(tt.get(ZobristHash(0)), Some(entry));
        assert_eq!(tt.occupied, 1);
    }

    #[test]
    fn set_and_get_different_slot() {
        let mut tt = TranspositionTable::new(1.try_into().unwrap());

        let entry = TranspositionEntry {
            flag: TranspositionFlag::Lowerbound,
            depth: 2,
            score: Score(1234),
            best_move: Some(MoveBuilder::new(Square::E2, Square::E4, Piece::WHITE_PAWN).build()),
        };

        tt.set(ZobristHash(0), entry.clone());

        assert_eq!(tt.get(ZobristHash(1)), None);
    }

    #[test]
    fn set_and_get_same_slot_different_hash() {
        let mut tt = TranspositionTable::new(1.try_into().unwrap());

        let entry = TranspositionEntry {
            flag: TranspositionFlag::Lowerbound,
            depth: 2,
            score: Score(1234),
            best_move: Some(MoveBuilder::new(Square::E2, Square::E4, Piece::WHITE_PAWN).build()),
        };

        tt.set(ZobristHash(0), entry.clone());

        assert_eq!(tt.get(ZobristHash(0x8000_0000_0000_0000)), None);
    }

    #[test]
    fn replace_same_slot_different_hash() {
        let mut tt = TranspositionTable::new(1.try_into().unwrap());

        let entry1 = TranspositionEntry {
            flag: TranspositionFlag::Lowerbound,
            depth: 2,
            score: Score(1234),
            best_move: Some(MoveBuilder::new(Square::E2, Square::E4, Piece::WHITE_PAWN).build()),
        };
        let entry2 = TranspositionEntry {
            flag: TranspositionFlag::Exact,
            depth: 3,
            score: Score(-123),
            best_move: Some(MoveBuilder::new(Square::G1, Square::F3, Piece::WHITE_KNIGHT).build()),
        };

        tt.set(ZobristHash(0), entry1.clone());
        // Replace!
        tt.set(ZobristHash(0x8000_0000_0000_0000), entry2.clone());

        assert_eq!(tt.get(ZobristHash(0)), None);
        assert_eq!(tt.get(ZobristHash(0x8000_0000_0000_0000)), Some(entry2));
        assert_eq!(tt.occupied, 1);
    }

    #[test]
    fn set_and_get_different_slots() {
        let mut tt = TranspositionTable::new(1.try_into().unwrap());

        let entry1 = TranspositionEntry {
            flag: TranspositionFlag::Lowerbound,
            depth: 2,
            score: Score(1234),
            best_move: Some(MoveBuilder::new(Square::E2, Square::E4, Piece::WHITE_PAWN).build()),
        };
        let entry2 = TranspositionEntry {
            flag: TranspositionFlag::Exact,
            depth: 3,
            score: Score(-123),
            best_move: Some(MoveBuilder::new(Square::G1, Square::F3, Piece::WHITE_KNIGHT).build()),
        };

        tt.set(ZobristHash(0), entry1.clone());
        tt.set(ZobristHash(1), entry2.clone());

        assert_eq!(tt.get(ZobristHash(0)), Some(entry1));
        assert_eq!(tt.get(ZobristHash(1)), Some(entry2));
        assert_eq!(tt.occupied, 2);
    }

    #[test]
    fn clear_resets_all_slots() {
        let mut tt = TranspositionTable::new(1.try_into().unwrap());

        let entry1 = TranspositionEntry {
            flag: TranspositionFlag::Lowerbound,
            depth: 2,
            score: Score(1234),
            best_move: Some(MoveBuilder::new(Square::E2, Square::E4, Piece::WHITE_PAWN).build()),
        };
        let entry2 = TranspositionEntry {
            flag: TranspositionFlag::Exact,
            depth: 3,
            score: Score(-123),
            best_move: Some(MoveBuilder::new(Square::G1, Square::F3, Piece::WHITE_KNIGHT).build()),
        };

        tt.set(ZobristHash(0), entry1.clone());
        tt.set(ZobristHash(1), entry2.clone());

        assert_eq!(tt.occupied, 2);

        tt.clear();

        assert_eq!(tt.get(ZobristHash(0)), None);
        assert_eq!(tt.get(ZobristHash(1)), None);
        assert_eq!(tt.occupied, 0);
    }

    #[test]
    fn resize_clears() {
        let mut tt = TranspositionTable::new(1.try_into().unwrap());

        let entry1 = TranspositionEntry {
            flag: TranspositionFlag::Lowerbound,
            depth: 2,
            score: Score(1234),
            best_move: Some(MoveBuilder::new(Square::E2, Square::E4, Piece::WHITE_PAWN).build()),
        };
        let entry2 = TranspositionEntry {
            flag: TranspositionFlag::Exact,
            depth: 3,
            score: Score(-123),
            best_move: Some(MoveBuilder::new(Square::G1, Square::F3, Piece::WHITE_KNIGHT).build()),
        };

        tt.set(ZobristHash(0), entry1.clone());
        tt.set(ZobristHash(1), entry2.clone());

        assert_eq!(tt.occupied, 2);

        tt.resize(1.try_into().unwrap());

        assert_eq!(tt.get(ZobristHash(0)), None);
        assert_eq!(tt.get(ZobristHash(1)), None);
        assert_eq!(tt.occupied, 0);
    }

    #[test]
    fn occupancy_reports_permille_occupied() {
        let mut tt = TranspositionTable::new(1.try_into().unwrap());

        tt.occupied = (tt.num_entries / 2) as _;
        assert_eq!(tt.occupancy(), 500);

        tt.occupied = (tt.num_entries / 3) as _;
        assert_eq!(tt.occupancy(), 333);
    }

    #[test]
    fn extract_pv_extracts_until_no_tt_entry() {
        // Arrange
        let mut board = Board::starting_position();
        let mut tt = TranspositionTable::new(1.try_into().unwrap());

        let default_entry = TranspositionEntry {
            flag: TranspositionFlag::Exact,
            depth: 5,
            score: Score(0),
            best_move: None,
        };

        let e4 = board.get_move(Square::E2, Square::E4, None).unwrap();
        tt.set(
            board.zobrist_hash(),
            TranspositionEntry {
                best_move: Some(e4),
                ..default_entry.clone()
            },
        );
        board.push_move_repr(e4);

        let e5 = board.get_move(Square::E7, Square::E5, None).unwrap();
        tt.set(
            board.zobrist_hash(),
            TranspositionEntry {
                best_move: Some(e5),
                ..default_entry.clone()
            },
        );
        board.push_move_repr(e5);

        let nf3 = board.get_move(Square::G1, Square::F3, None).unwrap();
        tt.set(
            board.zobrist_hash(),
            TranspositionEntry {
                best_move: Some(nf3),
                ..default_entry.clone()
            },
        );

        board = Board::starting_position();

        // Act
        let pv = tt.extract_pv(&mut board);

        // Assert
        assert_eq!(board, Board::starting_position());
        assert_eq!(pv, vec![e4, e5, nf3]);
    }

    #[test]
    fn extract_pv_extracts_until_no_replaced_entry() {
        // Arrange
        let mut board = Board::starting_position();
        let mut tt = TranspositionTable::new(1.try_into().unwrap());

        let default_entry = TranspositionEntry {
            flag: TranspositionFlag::Exact,
            depth: 5,
            score: Score(0),
            best_move: None,
        };

        let e4 = board.get_move(Square::E2, Square::E4, None).unwrap();
        tt.set(
            board.zobrist_hash(),
            TranspositionEntry {
                best_move: Some(e4),
                ..default_entry.clone()
            },
        );
        board.push_move_repr(e4);

        let e5 = board.get_move(Square::E7, Square::E5, None).unwrap();
        tt.set(
            board.zobrist_hash(),
            TranspositionEntry {
                best_move: Some(e5),
                ..default_entry.clone()
            },
        );
        board.push_move_repr(e5);

        let nf3 = board.get_move(Square::G1, Square::F3, None).unwrap();
        tt.set(
            // Deliberately create an entry in the same slot but with a different hash key
            board.zobrist_hash() ^ ZobristHash(0x8000_0000_0000_0000),
            TranspositionEntry {
                best_move: Some(nf3),
                ..default_entry.clone()
            },
        );
        // This assert is part of the setup - if we switch to using buckets so
        // each slot can hold more than one entry this assert will fail
        assert_eq!(tt.get(board.zobrist_hash()), None);

        board = Board::starting_position();

        // Act
        let pv = tt.extract_pv(&mut board);

        // Assert
        assert_eq!(board, Board::starting_position());
        assert_eq!(pv, vec![e4, e5]);
    }

    #[test]
    fn extract_pv_extracts_until_loop() {
        // Arrange
        let mut board = Board::starting_position();
        let mut tt = TranspositionTable::new(1.try_into().unwrap());

        let default_entry = TranspositionEntry {
            flag: TranspositionFlag::Exact,
            depth: 5,
            score: Score(0),
            best_move: None,
        };

        let moves = ["g1f3", "b8c6", "f3g1", "c6b8"];

        for m in moves {
            let m: UCIMove = m.parse().unwrap();
            let m = board.get_move(m.from, m.to, m.promotion).unwrap();
            tt.set(
                board.zobrist_hash(),
                TranspositionEntry {
                    best_move: Some(m),
                    ..default_entry.clone()
                },
            );
            board.push_move_repr(m);
        }

        assert_eq!(
            tt.get(board.zobrist_hash()),
            Some(TranspositionEntry {
                best_move: Some(
                    MoveBuilder::new(Square::G1, Square::F3, Piece::WHITE_KNIGHT).build()
                ),
                ..default_entry.clone()
            })
        );

        board = Board::starting_position();

        // Act
        let pv = tt.extract_pv(&mut board);

        // Assert
        assert_eq!(board, Board::starting_position());
        assert_eq!(
            pv,
            vec![
                MoveBuilder::new(Square::G1, Square::F3, Piece::WHITE_KNIGHT).build(),
                MoveBuilder::new(Square::B8, Square::C6, Piece::BLACK_KNIGHT).build(),
                MoveBuilder::new(Square::F3, Square::G1, Piece::WHITE_KNIGHT).build(),
                MoveBuilder::new(Square::C6, Square::B8, Piece::BLACK_KNIGHT).build(),
            ]
        );
    }
}
