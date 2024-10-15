mod killer_table;
mod see;

use hardfiskur_core::{
    board::{Board, Move, Piece},
    move_gen::MoveVec,
};

pub struct MoveOrderer {
    // killers: KillerTable,
}

impl MoveOrderer {
    pub fn new() -> Self {
        Self {
            // killers: KillerTable::default(),
        }
    }

    // pub fn store_killer(&mut self, ply_from_root: u16, m: Move) {
    //     self.killers.store(ply_from_root, m);
    // }

    // pub fn is_killer(&self, ply_from_root: u16, m: Move) -> bool {
    //     self.killers.is_killer(ply_from_root, m)
    // }
}

impl Default for MoveOrderer {
    fn default() -> Self {
        Self::new()
    }
}

impl MoveOrderer {
    const HASH_MOVE_SCORE: i32 = 100_000_000;
    const WINNING_CAPTURE_BIAS: i32 = 8_000_000;
    // const KILLER_BIAS: i32 = 4_000_000;
    const QUIET_BIAS: i32 = 0;
    // const LOSING_CAPTURE_BIAS: i32 = -2_000_000;

    pub fn order_moves(
        &self,
        _board: &Board,
        ply_from_root: u16,
        tt_move: Option<Move>,
        moves: MoveVec,
    ) -> OrderedMoves {
        // let seer = Seer::new(board);

        let scores = moves
            .iter()
            .map(|m| self.score_move(ply_from_root, tt_move, *m))
            .collect();
        OrderedMoves { moves, scores }
    }

    pub fn score_move(&self, _ply_from_root: u16, tt_move: Option<Move>, m: Move) -> i32 {
        if Some(m) == tt_move {
            Self::HASH_MOVE_SCORE
        } else if let Some(victim) = m.captured_piece() {
            let aggressor = m.piece();
            // Order by MVV-LVA
            Self::WINNING_CAPTURE_BIAS + self.mvv_lva_score(victim, aggressor)
        // } else if self.is_killer(ply_from_root, m) {
        //     Self::KILLER_BIAS
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

pub struct OrderedMoves {
    moves: MoveVec,
    scores: Vec<i32>,
}

impl Iterator for OrderedMoves {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        if self.scores.is_empty() {
            return None;
        }

        let mut max = self.scores[0];
        let mut max_idx = 0;

        for i in 1..self.scores.len() {
            if self.scores[i] > max {
                max = self.scores[i];
                max_idx = i;
            }
        }

        self.scores.swap_remove(max_idx);
        Some(self.moves.swap_remove(max_idx))
    }
}
