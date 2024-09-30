use std::{num::NonZeroU32, u32};

use hardfiskur_core::board::{Move, ZobristHash};
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
    key: u16,
    flag: TranspositionFlagInternal,
    depth: u32,
    score: Score,
    best_move: Option<NonZeroU32>,
}

impl TranspositionEntryInternal {
    fn key(hash: ZobristHash) -> u16 {
        (hash.0 >> 48) as u16
    }
}

#[derive(Debug, Clone, Default, FromZeroes)]
#[repr(align(64))]
struct TranspositionBucket {
    entries: [TranspositionEntryInternal; 4],
}

impl TranspositionBucket {
    pub fn find(&self, key: ZobristHash) -> Option<&TranspositionEntryInternal> {
        let key = TranspositionEntryInternal::key(key);

        for entry in self.entries.iter() {
            if entry.key == key {
                return Some(entry);
            }
        }

        None
    }

    pub fn store(&mut self, to_store: TranspositionEntryInternal) {
        let mut idx_lowest_depth = 0;

        for (i, entry) in self.entries.iter().enumerate() {
            if entry.depth < to_store.depth {
                idx_lowest_depth = i;
            }
        }

        self.entries[idx_lowest_depth] = to_store;
    }
}

pub struct TranspositionTable {
    hash_mask: usize,
    buckets: Vec<TranspositionBucket>,
}

impl TranspositionTable {
    pub fn new(max_size_in_mb: usize) -> Self {
        assert!(max_size_in_mb > 0);

        const BYTES_PER_MB: usize = 1024 * 1024;

        let entry_size = size_of::<TranspositionBucket>();
        let max_entries = max_size_in_mb * BYTES_PER_MB / entry_size;

        let num_entries = 1 << (usize::BITS - max_entries.leading_zeros() - 1);

        debug_assert!(
            size_of::<TranspositionBucket>() * num_entries <= max_size_in_mb * BYTES_PER_MB
        );

        Self {
            hash_mask: num_entries - 1,
            buckets: vec![FromZeroes::new_zeroed(); num_entries],
        }
    }

    pub fn index(&self, key: ZobristHash) -> usize {
        key.0 as usize & self.hash_mask
    }

    pub fn get_entry(&self, key: ZobristHash) -> Option<TranspositionEntry> {
        let index = self.index(key);
        let bucket = &self.buckets[index];

        bucket.find(key).and_then(|entry| {
            Some(TranspositionEntry {
                flag: entry.flag.try_into().ok()?,
                depth: entry.depth,
                score: entry.score,
                best_move: entry.best_move.map(|m| Move::from_nonzero(m)),
            })
        })
    }

    pub fn set(&mut self, key: ZobristHash, entry: TranspositionEntry) {
        let index = self.index(key);
        let bucket = &mut self.buckets[index];

        bucket.store(TranspositionEntryInternal {
            key: TranspositionEntryInternal::key(key),
            flag: entry.flag.into(),
            depth: entry.depth,
            score: entry.score,
            best_move: entry.best_move.map(|m| m.get()),
        });
    }
}
