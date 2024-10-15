// In practice, we should never get to this search depth; however it avoids
// pathlogical behavior if the search function has a bug that immediately
// returns, for example.
pub const MAX_DEPTH: i16 = 256;
pub const MAX_EXTENSIONS: i16 = 16;

// Reverse Futility Pruning parameters
pub const RFP_MAX_DEPTH: i16 = 6;
pub const RFP_MARGIN: i32 = 800;

// Null Move Pruning parameters
pub const NMP_MIN_DEPTH: i16 = 4;
pub const NMP_REDUCTION: i16 = 3;
