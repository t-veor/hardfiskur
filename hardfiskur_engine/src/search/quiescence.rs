use hardfiskur_core::move_gen::{MoveGenFlags, MoveVec};

use crate::{
    evaluation::evaluate,
    move_ordering::MovePicker,
    score::Score,
    transposition_table::{TranspositionEntry, TranspositionFlag},
};

use super::SearchContext;

impl<'a> SearchContext<'a> {
    pub fn quiescence(&mut self, ply_from_root: u16, mut alpha: Score, beta: Score) -> Score {
        self.consistency_check();

        // Increment stats
        self.stats.nodes_searched += 1;
        self.stats.quiescence_nodes += 1;
        self.stats.sel_depth = self.stats.sel_depth.max(ply_from_root);

        let (mut best_score, tt_entry) = if let Some(entry) = self.tt.get(self.board.zobrist_hash())
        {
            if Self::should_cutoff_quiescence(&entry, alpha, beta, ply_from_root) {
                self.stats.tt_hits += 1;

                return entry.get_score(ply_from_root);
            }

            (entry.get_score(ply_from_root), Some(entry))
        } else {
            // Score from standing pat.
            (evaluate(&self.board), None)
        };

        if best_score >= beta {
            // Beta cutoff!
            self.stats.beta_cutoffs += 1;
            return best_score;
        }

        alpha = alpha.max(best_score);

        let capturing_moves = {
            let mut moves = MoveVec::new();
            self.board
                .legal_moves_ex(MoveGenFlags::GEN_CAPTURES, &mut moves);

            moves
        };

        let mut ordered_moves =
            MovePicker::new(capturing_moves, tt_entry.and_then(|entry| entry.best_move));

        let mut best_move = None;
        while let Some(m) =
            ordered_moves.next_move(self.board, ply_from_root, &self.killers, self.history)
        {
            if !m.is_capture() {
                continue;
            }

            self.board.push_move_unchecked(m);

            let eval = -self.quiescence(ply_from_root + 1, -beta, -alpha);

            self.board.pop_move();

            if eval > best_score {
                best_score = eval;
                best_move = Some(m);

                if eval >= beta {
                    // Beta cutoff!
                    self.stats.beta_cutoffs += 1;
                    break;
                }
            }

            alpha = alpha.max(eval);
        }

        let flag = if best_score >= beta {
            TranspositionFlag::Lowerbound
        } else {
            TranspositionFlag::Upperbound
        };
        self.tt.set(
            self.board.zobrist_hash(),
            TranspositionEntry::new(flag, 0, best_score, best_move, ply_from_root),
        );

        best_score
    }

    fn should_cutoff_quiescence(
        entry: &TranspositionEntry,
        alpha: Score,
        beta: Score,
        ply_from_root: u16,
    ) -> bool {
        match entry.flag {
            TranspositionFlag::Exact => true,
            TranspositionFlag::Lowerbound => entry.get_score(ply_from_root) >= beta,
            TranspositionFlag::Upperbound => entry.get_score(ply_from_root) <= alpha,
        }
    }
}
