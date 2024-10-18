use zerocopy::FromZeros;
use zerocopy_derive::{FromBytes, IntoBytes};

pub trait Trace: Sized {
    fn add(&mut self, f: impl Fn(&mut EvalTrace));
}

#[derive(Debug, Clone, Copy)]
pub struct NullTrace;

impl Trace for NullTrace {
    fn add(&mut self, _f: impl Fn(&mut EvalTrace)) {}
}

#[derive(Debug, Clone, FromBytes, IntoBytes)]
#[repr(C)]
pub struct EvalTrace {
    pub material: [i32; 6],
    pub pawn_pst: [i32; 64],
    pub knight_pst: [i32; 64],
    pub bishop_pst: [i32; 64],
    pub rook_pst: [i32; 64],
    pub queen_pst: [i32; 64],
    pub king_pst: [i32; 64],
}

impl EvalTrace {
    pub const LEN: usize = std::mem::size_of::<EvalTrace>() / std::mem::size_of::<i32>();
}

impl Default for EvalTrace {
    fn default() -> Self {
        Self::new_zeroed()
    }
}
