#![cfg_attr(any(), rustfmt::skip)]

use crate::s;
use super::packed_score::S;

pub const MATERIAL: [S; 6] = [
    s!(105,160), s!(428,447), s!(471,464), s!(579,816), s!(1278,1441), s!(0), 
];

pub const PAWN_PST: [S; 64] = [
    s!(        0), s!(        0), s!(        0), s!(        0), s!(        0), s!(        0), s!(        0), s!(        0), 
    s!(  79, 255), s!(  86, 240), s!(  73, 233), s!( 109, 167), s!(  94, 160), s!(  72, 182), s!( -33, 246), s!( -63, 263), 
    s!(   0,  17), s!(  -2,  16), s!(  40,   4), s!(  54,   3), s!(  61,  -8), s!( 109, -14), s!(  61,  27), s!(  34,  10), 
    s!( -17,   6), s!(  -8,  -2), s!(   7,  -5), s!(  13, -26), s!(  33, -21), s!(  39, -19), s!(   4,  -7), s!(   2, -18), 
    s!( -34, -13), s!( -17, -12), s!(  -1, -16), s!(  26, -23), s!(  18, -19), s!(  24, -20), s!(  -4, -20), s!( -19, -30), 
    s!( -29, -20), s!( -23, -18), s!(  -2, -17), s!(   8, -11), s!(  19,  -7), s!( -14,  -6), s!( -10, -20), s!( -27, -32), 
    s!( -30, -15), s!( -19, -16), s!( -16,  -5), s!(  -3,  -4), s!(   3,   9), s!(  -8,   8), s!(  -6, -15), s!( -49, -24), 
    s!(        0), s!(        0), s!(        0), s!(        0), s!(        0), s!(        0), s!(        0), s!(        0), 
];
pub const KNIGHT_PST: [S; 64] = [
    s!(-228, -66), s!(-152, -17), s!( -83,   8), s!( -50,  -1), s!(  14,  -5), s!( -98, -24), s!(-142,  -9), s!(-152,-108), 
    s!( -52, -10), s!( -17,   9), s!(  51,   5), s!(  57,  14), s!(  57,  -3), s!( 114, -18), s!(   5,  -7), s!(  20, -38), 
    s!(  -5,  -3), s!(  44,  14), s!(  69,  52), s!(  91,  48), s!( 137,  27), s!( 161,  13), s!(  80,   2), s!(  42, -17), 
    s!(   6,  13), s!(  21,  39), s!(  53,  64), s!(  85,  66), s!(  56,  64), s!(  94,  58), s!(  31,  41), s!(  52,   0), 
    s!(  -9,  15), s!(   6,  30), s!(  31,  65), s!(  35,  66), s!(  48,  69), s!(  44,  50), s!(  40,  26), s!(   3,  10), 
    s!( -36,  -5), s!(  -4,  20), s!(  18,  31), s!(  24,  57), s!(  42,  53), s!(  28,  26), s!(  27,   8), s!(  -9,  -3), 
    s!( -41, -12), s!( -28,   3), s!( -10,  16), s!(  14,  16), s!(  15,  13), s!(  16,  10), s!(   2,  -8), s!(  -5,  -1), 
    s!( -93, -17), s!( -23, -25), s!( -42,  -1), s!( -17,   1), s!( -10,   2), s!(  -1,  -9), s!( -17, -22), s!( -60, -17), 
];
pub const BISHOP_PST: [S; 64] = [
    s!( -13,   9), s!( -64,  26), s!( -49,  19), s!( -99,  35), s!( -97,  30), s!( -62,  16), s!( -25,  11), s!( -70,  13), 
    s!( -23,   6), s!(  19,   9), s!(   5,  18), s!(  -8,  17), s!(  25,   7), s!(  33,   3), s!(   2,  18), s!(  11,  -8), 
    s!(   3,  26), s!(  35,  16), s!(  49,  21), s!(  60,  14), s!(  55,  17), s!(  84,  26), s!(  68,  12), s!(  48,  19), 
    s!(  -6,  23), s!(  21,  33), s!(  40,  26), s!(  62,  42), s!(  54,  36), s!(  44,  30), s!(  25,  25), s!(   9,  18), 
    s!(   2,  18), s!(  11,  29), s!(  23,  38), s!(  54,  37), s!(  50,  34), s!(  24,  31), s!(  18,  23), s!(  17,   5), 
    s!(  16,  17), s!(  28,  27), s!(  32,  32), s!(  31,  34), s!(  35,  38), s!(  37,  26), s!(  29,  18), s!(  38,   6), 
    s!(  23,  17), s!(  35,   7), s!(  36,   3), s!(  18,  21), s!(  32,  20), s!(  40,   9), s!(  64,   7), s!(  28,  -1), 
    s!(   9,   1), s!(  34,  13), s!(  21,   8), s!(   5,  19), s!(  10,  15), s!(   7,  27), s!(  27,  -3), s!(  30, -18), 
];
pub const ROOK_PST: [S; 64] = [
    s!(  21,  64), s!(  20,  66), s!(  24,  73), s!(  26,  66), s!(  48,  57), s!(  90,  46), s!(  86,  46), s!(  84,  47), 
    s!(  -1,  66), s!(  -6,  80), s!(  28,  76), s!(  55,  60), s!(  37,  62), s!(  92,  46), s!(  89,  42), s!( 113,  29), 
    s!( -17,  61), s!(  18,  57), s!(   9,  59), s!(  13,  53), s!(  53,  39), s!(  82,  29), s!( 142,  19), s!(  94,  18), 
    s!( -24,  61), s!(  -7,  55), s!(  -3,  61), s!(   3,  54), s!(   4,  42), s!(  31,  34), s!(  47,  33), s!(  41,  27), 
    s!( -31,  52), s!( -36,  55), s!( -25,  53), s!( -13,  49), s!( -10,  46), s!(  -5,  42), s!(  31,  28), s!(  11,  25), 
    s!( -30,  43), s!( -25,  40), s!( -18,  38), s!(  -9,  38), s!(   2,  32), s!(  16,  22), s!(  58,   0), s!(  26,   4), 
    s!( -27,  34), s!( -18,  37), s!(  -6,  39), s!(  -2,  35), s!(   8,  26), s!(  28,  16), s!(  50,   2), s!(  -6,  15), 
    s!(   6,  39), s!(   2,  37), s!(   8,  43), s!(  19,  33), s!(  27,  26), s!(  33,  29), s!(  31,  20), s!(  23,  12), 
];
pub const QUEEN_PST: [S; 64] = [
    s!(  -9, 147), s!( -12, 172), s!(  23, 187), s!(  69, 172), s!(  72, 169), s!(  84, 170), s!( 129, 103), s!(  32, 171), 
    s!(  15, 135), s!(  -7, 163), s!(   2, 204), s!( -18, 242), s!(  -8, 257), s!(  63, 206), s!(  37, 196), s!( 115, 152), 
    s!(  23, 129), s!(  18, 145), s!(  28, 181), s!(  42, 191), s!(  59, 217), s!( 104, 198), s!( 126, 151), s!( 111, 153), 
    s!(   8, 138), s!(  16, 161), s!(  21, 168), s!(  18, 202), s!(  24, 220), s!(  39, 213), s!(  43, 210), s!(  56, 183), 
    s!(  19, 135), s!(  15, 163), s!(  19, 163), s!(  23, 194), s!(  28, 186), s!(  30, 186), s!(  44, 176), s!(  47, 167), 
    s!(  21, 109), s!(  34, 125), s!(  31, 150), s!(  32, 143), s!(  34, 153), s!(  45, 153), s!(  57, 142), s!(  46, 129), 
    s!(  25, 107), s!(  34, 108), s!(  47, 101), s!(  50, 110), s!(  50, 114), s!(  62,  81), s!(  67,  55), s!(  68,  39), 
    s!(  28,  99), s!(  26,  99), s!(  39,  95), s!(  52, 100), s!(  45,  90), s!(  26,  97), s!(  27,  81), s!(  38,  60), 
];
pub const KING_PST: [S; 64] = [
    s!(  88,-137), s!(  58, -61), s!(  96, -48), s!( -40,   3), s!(   5, -13), s!(  35, -13), s!(  76, -21), s!( 177,-127), 
    s!( -86, -16), s!(  10,  21), s!( -30,  34), s!(  57,  24), s!(  10,  41), s!(  23,  54), s!(  25,  46), s!(  -4,   8), 
    s!(-100,   1), s!(  65,  28), s!( -38,  56), s!( -48,  67), s!( -31,  70), s!(  67,  60), s!(  46,  55), s!( -27,  18), 
    s!( -76, -11), s!( -77,  36), s!( -87,  63), s!(-141,  83), s!(-143,  83), s!(-113,  74), s!(-103,  59), s!(-141,  25), 
    s!( -87, -21), s!( -76,  20), s!(-118,  55), s!(-160,  81), s!(-165,  80), s!(-139,  60), s!(-141,  41), s!(-184,  20), 
    s!( -39, -32), s!(  -9,   1), s!( -88,  35), s!( -99,  53), s!( -96,  52), s!(-102,  38), s!( -56,  13), s!( -74,  -7), 
    s!(   5, -37), s!( -45,   2), s!(  10,   0), s!( -30,  16), s!( -28,  16), s!( -11,   4), s!( -36,   4), s!( -28, -20), 
    s!( -37, -75), s!( -13, -41), s!( -39, -14), s!( -28, -22), s!( -58, -24), s!(   4, -40), s!(  -9, -39), s!(  -7, -82), 
];

pub const PIECE_SQUARE_TABLES: [[S; 64]; 6] = [
    PAWN_PST, KNIGHT_PST, BISHOP_PST, ROOK_PST, QUEEN_PST, KING_PST
];

pub const KNIGHT_MOBILITY: [S; 9] = [
    s!(-35,-42), s!(-10,-9), s!(4,4), s!(8,10), s!(11,13), s!(11,16), s!(11,11), s!(6,3), s!(4,-7), 
];
pub const BISHOP_MOBILITY: [S; 14] = [
    s!(-32,-73), s!(-19,-44), s!(-8,-26), s!(-1,-9), s!(8,4), s!(10,17), s!(14,22), s!(16,28), s!(16,32), s!(19,28), s!(24,26), s!(31,21), s!(19,36), s!(63,0), 
];
pub const ROOK_MOBILITY: [S; 15] = [
    s!(-29,-51), s!(-18,-24), s!(-14,-21), s!(-7,-16), s!(-9,-6), s!(-2,-2), s!(1,2), s!(6,3), s!(9,8), s!(13,11), s!(17,13), s!(18,18), s!(24,19), s!(27,17), s!(35,14), 
];
pub const QUEEN_MOBILITY: [S; 28] = [
    s!(-44,-78), s!(-44,-139), s!(-50,-102), s!(-50,-56), s!(-49,-44), s!(-45,-29), s!(-42,-15), s!(-44,9), s!(-43,23), s!(-39,20), s!(-40,30), s!(-38,37), s!(-36,37), s!(-37,45), s!(-35,48), s!(-33,53), s!(-34,62), s!(-34,67), s!(-26,64), s!(-20,65), s!(-20,68), s!(10,48), s!(24,43), s!(17,43), s!(71,22), s!(188,-49), s!(239,-80), s!(580,-241), 
];

pub const PASSED_PAWNS: [S; 64] = [
    s!(        0), s!(        0), s!(        0), s!(        0), s!(        0), s!(        0), s!(        0), s!(        0), 
    s!(  15,  35), s!(   1,  30), s!(  11,  29), s!(  16,  26), s!(   9,  29), s!(   9,  30), s!(  -6,  31), s!(   2,  34), 
    s!(  37, 212), s!(  56, 204), s!(  33, 160), s!(  10, 110), s!(  11, 114), s!(  10, 142), s!( -33, 162), s!( -67, 202), 
    s!(  34, 114), s!(  31, 107), s!(  30,  69), s!(  24,  63), s!(   8,  57), s!(  20,  66), s!(   8,  96), s!( -11, 109), 
    s!(  19,  68), s!(   4,  58), s!( -20,  41), s!( -13,  30), s!( -19,  32), s!(  -9,  39), s!(  -7,  64), s!(  10,  62), 
    s!(   5,  20), s!( -11,  27), s!( -31,  18), s!( -32,   9), s!( -22,   6), s!(  -2,   7), s!(   8,  33), s!(  28,  17), 
    s!(   0,  18), s!(   2,  22), s!( -20,  16), s!( -29,   6), s!(  -1, -11), s!(  13,  -6), s!(  40,   9), s!(  17,  18), 
    s!(        0), s!(        0), s!(        0), s!(        0), s!(        0), s!(        0), s!(        0), s!(        0), 
];

pub const DOUBLED_PAWNS: S = s!(  -9, -30);
pub const ISOLATED_PAWNS: S = s!( -20, -16);

pub const PAWN_SHIELD_CLOSE: S = s!(58,-25);
pub const PAWN_SHIELD_FAR: S = s!(40,-10);

pub const SEMI_OPEN_FILE_BONUSES: [S; 3] = [
    s!(18,19), s!(6,21), s!(-35,27), 
];
pub const OPEN_FILE_BONUSES: [S; 3] = [
    s!(47,13), s!(-16,41), s!(-100,0), 
];
