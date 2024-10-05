mod killer_table;
mod see;

use hardfiskur_core::board::Move;
use killer_table::KillerTable;

pub struct MoveOrderer {
    killers: KillerTable,
}

impl MoveOrderer {
    pub fn new() -> Self {
        Self {
            killers: KillerTable::default(),
        }
    }

    pub fn store_killer(&mut self, ply_from_root: u32, m: Move) {
        self.killers.store(ply_from_root, m);
    }

    pub fn is_killer(&self, ply_from_root: u32, m: Move) -> bool {
        self.killers.is_killer(ply_from_root, m)
    }
}

impl MoveOrderer {
    const HASH_MOVE_SCORE: i32 = 100_000_000;
    // const WINNING_CAPTURE_BIAS: i32 = 8_000_000;
    const CAPTURE_BIAS: i32 = 8_000_000;
    const KILLER_BIAS: i32 = 4_000_000;
    // const LOSING_CAPTURE_BIAS: i32 = 2_000_000;
    const QUIET_BIAS: i32 = 0;

    pub fn order_moves(&self, ply_from_root: u32, tt_move: Option<Move>, moves: &mut [Move]) {
        moves.sort_by_cached_key(|m| -self.score_move(ply_from_root, tt_move, *m));
    }

    pub fn score_move(&self, ply_from_root: u32, tt_move: Option<Move>, m: Move) -> i32 {
        if Some(m) == tt_move {
            Self::HASH_MOVE_SCORE
        } else if let Some(captured) = m.captured_piece() {
            // Order by Most Valuable Victim - Least Valuable Aggressor (MVV-LVA)
            Self::CAPTURE_BIAS
                // Most valuable victim (* 10 to make sure it's always
                // considered above the aggressor)
                + (captured.piece_type() as i32) * 10
                // Least valuable aggressor
                - (m.piece().piece_type() as i32)
        } else if self.is_killer(ply_from_root, m) {
            Self::KILLER_BIAS
        } else {
            Self::QUIET_BIAS
        }
    }
}
