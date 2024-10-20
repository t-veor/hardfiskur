use std::fmt::Display;

use zerocopy::FromZeros;
use zerocopy_derive::{FromBytes, Immutable, IntoBytes};

use super::{packed_score::PackedScore, parameters::*};

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
    pub material: [i16; 6],

    pub pawn_pst: [i16; 64],
    pub knight_pst: [i16; 64],
    pub bishop_pst: [i16; 64],
    pub rook_pst: [i16; 64],
    pub queen_pst: [i16; 64],
    pub king_pst: [i16; 64],

    pub knight_mobility: [i16; 9],
    pub bishop_mobility: [i16; 14],
    pub rook_mobility: [i16; 15],
    pub queen_mobility: [i16; 28],

    pub passed_pawns: [i16; 64],
}

impl EvalTrace {
    pub const LEN: usize = std::mem::size_of::<EvalTrace>() / std::mem::size_of::<i16>();
}

impl Default for EvalTrace {
    fn default() -> Self {
        Self::new_zeroed()
    }
}

impl Trace for EvalTrace {
    fn add(&mut self, f: impl Fn(&mut EvalTrace)) {
        f(self)
    }
}

pub type Parameter = [f64; 2];

#[derive(Debug, Clone, FromBytes, IntoBytes, Immutable)]
#[repr(C)]
pub struct EvalParameters {
    pub material: [Parameter; 6],

    pub pawn_pst: [Parameter; 64],
    pub knight_pst: [Parameter; 64],
    pub bishop_pst: [Parameter; 64],
    pub rook_pst: [Parameter; 64],
    pub queen_pst: [Parameter; 64],
    pub king_pst: [Parameter; 64],

    pub knight_mobility: [Parameter; 9],
    pub bishop_mobility: [Parameter; 14],
    pub rook_mobility: [Parameter; 15],
    pub queen_mobility: [Parameter; 28],

    pub passed_pawns: [Parameter; 64],
}

impl EvalParameters {
    pub const LEN: usize = std::mem::size_of::<EvalParameters>() / std::mem::size_of::<Parameter>();
}

const _: () = assert!(EvalTrace::LEN == EvalParameters::LEN);

impl EvalParameters {
    fn fmt_param(
        f: &mut std::fmt::Formatter<'_>,
        param: Parameter,
        pad_size: Option<usize>,
    ) -> std::fmt::Result {
        let [mg, eg] = param;
        let (mg, eg) = (mg.round() as i32, eg.round() as i32);

        let (single_width, double_width) = match pad_size {
            _ if !f.alternate() => (0, 0),
            Some(single_width) => (single_width, single_width * 2 + 1),
            None => (0, 0),
        };

        if mg == 0 && eg == 0 {
            write!(f, "s!({mg:>width$})", width = double_width)
        } else {
            write!(f, "s!({mg:>width$},{eg:>width$})", width = single_width)
        }
    }

    #[allow(dead_code)]
    fn fmt_single(
        f: &mut std::fmt::Formatter<'_>,
        name: &str,
        param: Parameter,
        pad_size: Option<usize>,
    ) -> std::fmt::Result {
        write!(f, "pub const {name}: S = ")?;
        Self::fmt_param(f, param, pad_size)?;
        write!(f, ";")?;

        if f.alternate() {
            writeln!(f)?;
        }

        Ok(())
    }

    fn fmt_array(
        f: &mut std::fmt::Formatter<'_>,
        name: &str,
        params: &[Parameter],
        pad_size: Option<usize>,
    ) -> std::fmt::Result {
        let size = params.len();
        write!(f, "pub const {name}: [S; {size}] = [")?;

        if f.alternate() {
            writeln!(f)?;
            write!(f, "    ")?;
        }
        for &param in params {
            Self::fmt_param(f, param, pad_size)?;
            write!(f, ", ")?;
        }

        if f.alternate() {
            writeln!(f)?;
        }

        write!(f, "];")?;

        if f.alternate() {
            writeln!(f)?;
        }

        Ok(())
    }

    fn fmt_pst(
        f: &mut std::fmt::Formatter<'_>,
        name: &str,
        params: &[Parameter; 64],
        pad_size: Option<usize>,
    ) -> std::fmt::Result {
        if !f.alternate() {
            return Self::fmt_array(f, name, params, pad_size);
        }

        writeln!(f, "pub const {name}: [S; 64] = [")?;

        for rank in 0..8 {
            // NOTE: rank is from black's perspective here!
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

            knight_mobility: convert_packed_score_array(KNIGHT_MOBILITY),
            bishop_mobility: convert_packed_score_array(BISHOP_MOBILITY),
            rook_mobility: convert_packed_score_array(ROOK_MOBILITY),
            queen_mobility: convert_packed_score_array(QUEEN_MOBILITY),

            passed_pawns: convert_packed_score_array(PASSED_PAWNS),
        }
    }
}

impl Display for EvalParameters {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let pad_size = Some(4);

        Self::fmt_array(f, "MATERIAL", &self.material, None)?;
        if f.alternate() {
            writeln!(f)?;
        }

        Self::fmt_pst(f, "PAWN_PST", &self.pawn_pst, pad_size)?;
        Self::fmt_pst(f, "KNIGHT_PST", &self.knight_pst, pad_size)?;
        Self::fmt_pst(f, "BISHOP_PST", &self.bishop_pst, pad_size)?;
        Self::fmt_pst(f, "ROOK_PST", &self.rook_pst, pad_size)?;
        Self::fmt_pst(f, "QUEEN_PST", &self.queen_pst, pad_size)?;
        Self::fmt_pst(f, "KING_PST", &self.king_pst, pad_size)?;

        if f.alternate() {
            writeln!(f)?;
        }

        write!(f, "pub const PIECE_SQUARE_TABLES: [[S; 64]; 6] = [")?;
        if f.alternate() {
            writeln!(f)?;
        }
        write!(
            f,
            "    PAWN_PST, KNIGHT_PST, BISHOP_PST, ROOK_PST, QUEEN_PST, KING_PST"
        )?;
        if f.alternate() {
            writeln!(f)?;
        }
        write!(f, "];")?;
        if f.alternate() {
            writeln!(f)?;
            writeln!(f)?;
        }

        Self::fmt_array(f, "KNIGHT_MOBILITY", &self.knight_mobility, None)?;
        Self::fmt_array(f, "BISHOP_MOBILITY", &self.bishop_mobility, None)?;
        Self::fmt_array(f, "ROOK_MOBILITY", &self.rook_mobility, None)?;
        Self::fmt_array(f, "QUEEN_MOBILITY", &self.queen_mobility, None)?;

        if f.alternate() {
            writeln!(f)?;
        }

        Self::fmt_pst(f, "PASSED_PAWNS", &self.passed_pawns, pad_size)?;

        Ok(())
    }
}

impl From<PackedScore> for Parameter {
    fn from(value: PackedScore) -> Self {
        [value.mg() as f64, value.eg() as f64]
    }
}

fn convert_packed_score_array<const N: usize>(array: [PackedScore; N]) -> [Parameter; N] {
    array.map(|x| x.into())
}
