use hardfiskur_core::board::PieceType;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Phase(pub i32);

impl Phase {
    pub const FULL_ENDGAME_PHASE: i32 = 24;

    pub fn phase_modifier(piece_type: impl Into<PieceType>) -> i32 {
        match piece_type.into() {
            PieceType::Knight => 1,
            PieceType::Bishop => 1,
            PieceType::Rook => 2,
            PieceType::Queen => 4,
            _ => 0,
        }
    }

    pub fn apply_phase(&mut self, piece_type: impl Into<PieceType>) {
        self.0 += Self::phase_modifier(piece_type);
    }

    pub fn taper(&self, midgame_eval: i32, endgame_eval: i32) -> i32 {
        (midgame_eval * self.0 + endgame_eval * (Self::FULL_ENDGAME_PHASE - self.0))
            / Self::FULL_ENDGAME_PHASE
    }
}
