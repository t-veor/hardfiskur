use std::fmt::Display;

use zerocopy::FromZeros;
use zerocopy_derive::{FromBytes, IntoBytes};

use super::{
    packed_score::PackedScore,
    parameters::{BISHOP_PST, KING_PST, KNIGHT_PST, MATERIAL, PAWN_PST, QUEEN_PST, ROOK_PST},
};

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

pub type Parameter = (f32, f32);

pub struct EvalParameters {
    pub material: [Parameter; 6],
    pub pawn_pst: [Parameter; 64],
    pub knight_pst: [Parameter; 64],
    pub bishop_pst: [Parameter; 64],
    pub rook_pst: [Parameter; 64],
    pub queen_pst: [Parameter; 64],
    pub king_pst: [Parameter; 64],
}

impl EvalParameters {
    fn fmt_param(
        f: &mut std::fmt::Formatter<'_>,
        param: Parameter,
        pad_size: Option<usize>,
    ) -> std::fmt::Result {
        let (mg, eg) = param;
        let (mg, eg) = (mg.round() as i32, eg.round() as i32);

        let (single_width, double_width) = match pad_size {
            Some(single_width) => (single_width, single_width * 2 + 2),
            None => (0, 0),
        };

        if mg == 0 && eg == 0 {
            write!(f, "s!({mg:>width$})", width = double_width)
        } else {
            write!(f, "s!({mg:>width$}, {eg:>width$})", width = single_width)
        }
    }

    fn fmt_single(
        f: &mut std::fmt::Formatter<'_>,
        name: &str,
        param: Parameter,
        pad_size: Option<usize>,
    ) -> std::fmt::Result {
        write!(f, "pub const {name}: S = ")?;
        Self::fmt_param(f, param, pad_size)?;
        writeln!(f, ";")
    }

    fn fmt_array(
        f: &mut std::fmt::Formatter<'_>,
        name: &str,
        params: &[Parameter],
        pad_size: Option<usize>,
    ) -> std::fmt::Result {
        let size = params.len();
        writeln!(f, "pub const {name}: [S; {size}] = [")?;
        write!(f, "    ")?;
        for &param in params {
            Self::fmt_param(f, param, pad_size)?;
            write!(f, ", ")?;
        }

        writeln!(f)?;
        writeln!(f, "];")
    }

    fn fmt_pst(
        f: &mut std::fmt::Formatter<'_>,
        name: &str,
        params: &[Parameter; 64],
        pad_size: Option<usize>,
    ) -> std::fmt::Result {
        writeln!(f, "pub const {name}: [S; 64] = [")?;

        for rank in 0..8 {
            write!(f, "    ")?;
            for file in 0..8 {
                let i = rank * 8 + file;
                let param = params[i];
                Self::fmt_param(f, param, pad_size)?;
                write!(f, ", ")?;
            }
            writeln!(f)?;
        }

        writeln!(f, "];")
    }
}

impl Default for EvalParameters {
    fn default() -> Self {
        Self {
            material: convert_packed_score_array(MATERIAL),
            pawn_pst: convert_packed_score_array(PAWN_PST),
            knight_pst: convert_packed_score_array(KNIGHT_PST),
            bishop_pst: convert_packed_score_array(BISHOP_PST),
            rook_pst: convert_packed_score_array(ROOK_PST),
            queen_pst: convert_packed_score_array(QUEEN_PST),
            king_pst: convert_packed_score_array(KING_PST),
        }
    }
}

impl Display for EvalParameters {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let pad_size = Some(5);

        Self::fmt_array(f, "MATERIAL", &self.material, None)?;
        writeln!(f)?;

        Self::fmt_pst(f, "PAWN_PST", &self.pawn_pst, pad_size)?;
        Self::fmt_pst(f, "KNIGHT_PST", &self.knight_pst, pad_size)?;
        Self::fmt_pst(f, "BISHOP_PST", &self.bishop_pst, pad_size)?;
        Self::fmt_pst(f, "ROOK_PST", &self.rook_pst, pad_size)?;
        Self::fmt_pst(f, "QUEEN_PST", &self.queen_pst, pad_size)?;
        Self::fmt_pst(f, "KING_PST", &self.king_pst, pad_size)?;
        writeln!(f)?;

        writeln!(f, "pub const PIECE_SQUARE_TABLES: [[S; 64]; 6] = [")?;
        writeln!(
            f,
            "    PAWN_PST, KNIGHT_PST, BISHOP_PST, ROOK_PST, QUEEN_PST, KING_PST"
        )?;
        writeln!(f, "];")?;

        Ok(())
    }
}

impl From<PackedScore> for Parameter {
    fn from(value: PackedScore) -> Self {
        (value.mg() as f32, value.eg() as f32)
    }
}

fn convert_packed_score_array<const N: usize>(array: [PackedScore; N]) -> [Parameter; N] {
    array.map(|x| x.into())
}
