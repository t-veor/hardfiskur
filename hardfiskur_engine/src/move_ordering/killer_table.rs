use hardfiskur_core::board::Move;

#[derive(Default)]
pub struct KillerTable {
    buckets: [KillerTableBucket; 32],
}

#[derive(Default)]
struct KillerTableBucket {
    killers: [Option<Move>; 2],
}

impl KillerTable {
    pub fn store(&mut self, ply_from_root: u32, m: Move) {
        if let Some(bucket) = self.buckets.get_mut(ply_from_root as usize) {
            bucket.store(m)
        }
    }

    pub fn is_killer(&self, ply_from_root: u32, m: Move) -> bool {
        self.buckets
            .get(ply_from_root as usize)
            .map(|bucket| bucket.contains(m))
            .unwrap_or(false)
    }
}

impl KillerTableBucket {
    fn store(&mut self, m: Move) {
        // This routine inserts m into the start of the bucket, shifting the
        // remaining entries up and discarding the last entry.

        // e.g. consider m = 0, killers = [1, 2, 3]
        // 1st iteration: swap m with killers[0] -> m = 1, killers = [0, 2, 3]
        // 2nd iteration: swap m with killers[1] -> m = 2, killers = [0, 1, 3]
        // 3nd iteration: swap m with killers[2] -> m = 3, killers = [0, 1, 2]

        let mut tmp = Some(m);
        for entry in self.killers.iter_mut() {
            std::mem::swap(&mut tmp, entry);
        }
    }

    fn contains(&self, m: Move) -> bool {
        self.killers.iter().any(|i| *i == Some(m))
    }
}
