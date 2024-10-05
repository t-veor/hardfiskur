mod killer_table;
mod see;

use hardfiskur_core::board::{Board, Move, Piece, PieceType};
use killer_table::KillerTable;
use see::Seer;

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
    const WINNING_CAPTURE_BIAS: i32 = 8_000_000;
    const KILLER_BIAS: i32 = 4_000_000;
    const QUIET_BIAS: i32 = 0;
    const LOSING_CAPTURE_BIAS: i32 = -2_000_000;

    pub fn order_moves(
        &self,
        board: &Board,
        ply_from_root: u32,
        tt_move: Option<Move>,
        moves: &mut [Move],
    ) {
        let seer = Seer::new(board);

        moves.sort_by_cached_key(|m| -self.score_move(ply_from_root, tt_move, &seer, *m));
    }

    pub fn score_move(
        &self,
        ply_from_root: u32,
        tt_move: Option<Move>,
        seer: &Seer,
        m: Move,
    ) -> i32 {
        if Some(m) == tt_move {
            Self::HASH_MOVE_SCORE
        } else if let Some(victim) = m.captured_piece() {
            let aggressor = m.piece();
            // Is the capture actually good?
            let bias = match seer.see(m.from_square(), aggressor, m.to_square(), victim, 1) {
                true => Self::WINNING_CAPTURE_BIAS,
                false => Self::LOSING_CAPTURE_BIAS,
            };
            // Order by MVV-LVA next
            bias + self.mvv_lva_score(victim, aggressor)
        } else if self.is_killer(ply_from_root, m) {
            Self::KILLER_BIAS
        } else {
            Self::QUIET_BIAS
        }
    }

    fn mvv_lva_score(&self, victim: Piece, aggressor: Piece) -> i32 {
        // Most valuable victim (* 10 to make sure it's always considered above
        // the aggressor)
        (victim.piece_type() as i32) * 10
        // Least valuable aggressor
        - (aggressor.piece_type() as i32)
    }
}
