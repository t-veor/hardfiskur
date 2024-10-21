// In practice, we should never get to this search depth; however it avoids
// pathlogical behavior if the search function has a bug that immediately
// returns, for example.
pub const MAX_DEPTH: i16 = 256;
pub const MAX_EXTENSIONS: i16 = 16;

// Reverse Futility Pruning parameters
pub const RFP_MAX_DEPTH: i16 = 6;
pub const RFP_MARGIN: i32 = 80;

// Null Move Pruning parameters
pub const NMP_MIN_DEPTH: i16 = 4;
pub const NMP_REDUCTION: i16 = 3;

// Late Move Reduction parameters
pub const LMR_MIN_MOVES_PLAYED: usize = 3;
pub const LMR_MIN_DEPTH: i16 = 3;
pub const LMR_BASE: f64 = 0.77;
pub const LMR_DIVISOR: f64 = 2.36;

// Late Move Pruning parameters
pub const LMP_MAX_DEPTH: i16 = 4;
// A value of 3 here results in the following no. of quiets checked before
// giving up:
// Depth:           1   2   3   4
// Quiets to check: 3   5   7  11
pub const LMP_MARGIN: i32 = 3;

// Futility Pruning parameters
pub const FP_MAX_DEPTH: i16 = 5;
pub const FP_MARGIN: i32 = 150;
pub const FP_MARGIN_BASE: i32 = 50;
