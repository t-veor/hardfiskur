// In practice, we should never get to this search depth; however it avoids
// pathlogical behavior if the search function has a bug that immediately
// returns, for example.
pub const MAX_DEPTH: i16 = 256;
pub const MAX_EXTENSIONS: i16 = 16;
