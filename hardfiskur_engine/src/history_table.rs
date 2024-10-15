use hardfiskur_core::board::{Color, Move};

pub const MAX_HISTORY: i32 = (i16::MAX / 2) as i32;
pub const MAX_BONUS: i32 = MAX_HISTORY / 8;

type ButterflyTable<T> = [T; 64 * 64];

pub struct HistoryTable {
    quiets: [ButterflyTable<i32>; 2],
}

impl HistoryTable {
    pub fn new() -> Self {
        Self {
            quiets: [[0; 64 * 64]; 2],
        }
    }

    pub fn update_quiets(
        &mut self,
        to_move: Color,
        depth: i16,
        move_to_reward: Move,
        moves_to_penalize: &[Move],
    ) {
        let bonus = Self::bonus(depth);
        let color = to_move.is_black() as usize;

        Self::apply_bonus(
            &mut self.quiets[color][move_to_reward.butterfly_index()],
            bonus,
        );

        for &move_to_penalize in moves_to_penalize {
            Self::apply_bonus(
                &mut self.quiets[color][move_to_penalize.butterfly_index()],
                -bonus,
            );
        }
    }

    pub fn get_quiet_history(&self, to_move: Color, m: Move) -> i32 {
        self.quiets[to_move.is_black() as usize][m.butterfly_index()]
    }

    fn bonus(depth: i16) -> i32 {
        (300 * (depth as i32) - 300).clamp(0, MAX_BONUS)
    }

    fn apply_bonus(target: &mut i32, bonus: i32) {
        *target += bonus - *target * bonus.abs() / MAX_HISTORY;
    }

    pub fn clear(&mut self) {
        for table in self.quiets.iter_mut() {
            table.fill(0);
        }
    }
}

impl Default for HistoryTable {
    fn default() -> Self {
        Self::new()
    }
}
