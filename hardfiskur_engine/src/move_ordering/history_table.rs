use hardfiskur_core::board::Move;

pub const MAX_HISTORY_VALUE: i32 = 512;

pub struct HistoryTable {
    table: [[i32; 64]; 16],
}

impl Default for HistoryTable {
    fn default() -> Self {
        Self {
            table: [[0; 64]; 16],
        }
    }
}

impl HistoryTable {
    pub fn get_history_value(&self, m: Move) -> i32 {
        let piece = m.piece().get();
        let to = m.to_square();

        self.table[piece as usize][to.index()]
    }

    pub fn apply_bonus(&mut self, m: Move, bonus: i32) {
        let piece = m.piece().get();
        let to = m.to_square();
        let entry = &mut self.table[piece as usize][to.index()];

        let clamped_bonus = bonus.clamp(-MAX_HISTORY_VALUE, MAX_HISTORY_VALUE);
        *entry += clamped_bonus - *entry * clamped_bonus.abs() / MAX_HISTORY_VALUE;
    }
}
