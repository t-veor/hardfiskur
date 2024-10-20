#![cfg_attr(any(), rustfmt::skip)]

use crate::s;
use super::packed_score::S;

pub const MATERIAL: [S; 6] = [
    s!(117, 157), s!(417, 445), s!(458, 460), s!(587, 817), s!(1237, 1453), s!(0), 
];

pub const PAWN_PST: [S; 64] = [
    s!(0), s!(0), s!(0), s!(0), s!(0), s!(0), s!(0), s!(0), s!(57, 244), s!(77, 232), s!(57, 225), s!(86, 160), s!(77, 152), s!(55, 174), s!(-33, 236), s!(-71, 252), s!(-24, 28), s!(-3, 25), s!(20, -8), s!(28, 4), s!(42, -23), s!(85, -25), s!(75, 21), s!(25, 8), s!(-35, 18), s!(-8, 10), s!(-12, -12), s!(-9, -30), s!(17, -29), s!(13, -26), s!(20, -4), s!(-3, -15), s!(-47, 0), s!(-12, 5), s!(-15, -18), s!(7, -22), s!(8, -23), s!(0, -22), s!(9, -12), s!(-29, -24), s!(-39, -6), s!(-17, -1), s!(-15, -21), s!(-8, -10), s!(12, -12), s!(-4, -18), s!(32, -15), s!(-4, -31), s!(-35, -2), s!(-9, 1), s!(-22, -12), s!(-16, 0), s!(0, 3), s!(23, -12), s!(52, -18), s!(-9, -33), s!(0), s!(0), s!(0), s!(0), s!(0), s!(0), s!(0), s!(0), 
];
pub const KNIGHT_PST: [S; 64] = [
    s!(-219, -65), s!(-152, -18), s!(-74, 6), s!(-41, -2), s!(21, -7), s!(-89, -24), s!(-136, -9), s!(-136, -109), s!(-46, -9), s!(-10, 9), s!(54, 6), s!(54, 14), s!(54, -3), s!(119, -16), s!(3, -1), s!(20, -34), s!(-2, -3), s!(45, 14), s!(70, 51), s!(90, 48), s!(140, 26), s!(152, 17), s!(82, 3), s!(39, -13), s!(5, 14), s!(20, 41), s!(50, 64), s!(84, 65), s!(54, 65), s!(89, 59), s!(30, 41), s!(51, 2), s!(-11, 17), s!(5, 29), s!(29, 64), s!(34, 65), s!(47, 69), s!(41, 49), s!(33, 26), s!(-2, 10), s!(-36, -7), s!(-7, 18), s!(17, 31), s!(23, 57), s!(39, 52), s!(26, 24), s!(21, 8), s!(-12, -5), s!(-45, -12), s!(-28, 2), s!(-11, 13), s!(14, 15), s!(13, 13), s!(13, 9), s!(-6, -9), s!(-13, -1), s!(-101, -16), s!(-18, -23), s!(-40, -3), s!(-14, 2), s!(-9, 0), s!(-4, -10), s!(-15, -21), s!(-67, -21), 
];
pub const BISHOP_PST: [S; 64] = [
    s!(-13, 11), s!(-65, 27), s!(-47, 20), s!(-100, 36), s!(-96, 30), s!(-55, 16), s!(-19, 10), s!(-61, 15), s!(-15, 6), s!(22, 11), s!(5, 18), s!(-9, 16), s!(25, 8), s!(33, 4), s!(16, 17), s!(16, -9), s!(4, 26), s!(38, 16), s!(49, 20), s!(61, 13), s!(51, 16), s!(86, 24), s!(65, 14), s!(51, 18), s!(-3, 23), s!(18, 33), s!(42, 25), s!(59, 42), s!(54, 34), s!(43, 29), s!(22, 26), s!(5, 19), s!(-1, 19), s!(12, 28), s!(19, 36), s!(52, 35), s!(48, 32), s!(21, 31), s!(14, 23), s!(12, 6), s!(18, 16), s!(22, 27), s!(31, 30), s!(27, 31), s!(33, 36), s!(33, 26), s!(25, 16), s!(34, 7), s!(19, 17), s!(34, 8), s!(30, 2), s!(18, 21), s!(30, 18), s!(40, 9), s!(56, 13), s!(25, -2), s!(9, 3), s!(30, 14), s!(24, 10), s!(6, 19), s!(12, 16), s!(7, 26), s!(33, -5), s!(21, -11), 
];
pub const ROOK_PST: [S; 64] = [
    s!(41, 55), s!(27, 62), s!(33, 71), s!(35, 66), s!(60, 55), s!(91, 44), s!(74, 47), s!(96, 40), s!(15, 59), s!(7, 75), s!(42, 74), s!(72, 60), s!(53, 61), s!(92, 46), s!(89, 40), s!(119, 24), s!(-11, 59), s!(27, 56), s!(20, 59), s!(29, 54), s!(66, 38), s!(78, 31), s!(143, 17), s!(104, 13), s!(-23, 60), s!(-4, 55), s!(0, 64), s!(6, 58), s!(10, 43), s!(21, 36), s!(42, 32), s!(43, 24), s!(-35, 52), s!(-36, 55), s!(-25, 56), s!(-12, 53), s!(-8, 49), s!(-19, 46), s!(17, 30), s!(-4, 28), s!(-35, 44), s!(-28, 42), s!(-16, 41), s!(-9, 43), s!(0, 37), s!(1, 27), s!(40, 4), s!(11, 7), s!(-36, 37), s!(-23, 40), s!(-1, 39), s!(1, 39), s!(9, 29), s!(15, 21), s!(35, 10), s!(-13, 22), s!(-5, 34), s!(-2, 35), s!(15, 40), s!(24, 35), s!(30, 26), s!(24, 27), s!(28, 19), s!(6, 11), 
];
pub const QUEEN_PST: [S; 64] = [
    s!(-1, 145), s!(-18, 177), s!(21, 193), s!(61, 181), s!(65, 178), s!(82, 169), s!(105, 107), s!(36, 162), s!(23, 128), s!(-6, 163), s!(0, 207), s!(-16, 241), s!(-9, 260), s!(59, 208), s!(33, 189), s!(107, 149), s!(28, 125), s!(23, 143), s!(28, 183), s!(40, 195), s!(53, 221), s!(109, 190), s!(122, 144), s!(118, 140), s!(10, 139), s!(15, 161), s!(17, 174), s!(16, 202), s!(21, 221), s!(36, 214), s!(39, 204), s!(53, 179), s!(17, 136), s!(14, 164), s!(16, 165), s!(24, 195), s!(28, 187), s!(26, 184), s!(38, 172), s!(42, 164), s!(18, 114), s!(32, 126), s!(27, 154), s!(29, 147), s!(31, 159), s!(42, 152), s!(50, 140), s!(42, 127), s!(26, 113), s!(32, 111), s!(44, 105), s!(51, 110), s!(48, 118), s!(63, 82), s!(68, 51), s!(79, 34), s!(30, 101), s!(31, 98), s!(43, 96), s!(56, 105), s!(48, 95), s!(28, 97), s!(45, 72), s!(43, 65), 
];
pub const KING_PST: [S; 64] = [
    s!(68, -132), s!(39, -59), s!(73, -45), s!(-58, 4), s!(-6, -10), s!(39, -8), s!(76, -15), s!(170, -124), s!(-99, -12), s!(-14, 25), s!(-53, 37), s!(30, 24), s!(-7, 42), s!(10, 58), s!(11, 50), s!(-21, 13), s!(-116, 5), s!(46, 30), s!(-60, 57), s!(-75, 67), s!(-54, 71), s!(52, 63), s!(30, 61), s!(-42, 24), s!(-93, -6), s!(-100, 41), s!(-111, 66), s!(-171, 83), s!(-172, 85), s!(-121, 78), s!(-110, 63), s!(-155, 30), s!(-103, -16), s!(-98, 25), s!(-147, 61), s!(-198, 84), s!(-192, 84), s!(-148, 66), s!(-143, 46), s!(-189, 27), s!(-49, -25), s!(-32, 9), s!(-113, 42), s!(-132, 58), s!(-124, 59), s!(-118, 46), s!(-60, 21), s!(-83, 3), s!(60, -45), s!(5, -9), s!(-20, 10), s!(-69, 25), s!(-70, 28), s!(-42, 17), s!(24, -10), s!(33, -36), s!(45, -94), s!(91, -70), s!(64, -42), s!(-75, -16), s!(12, -45), s!(-38, -20), s!(65, -59), s!(59, -100), 
];

pub const PIECE_SQUARE_TABLES: [[S; 64]; 6] = [
    PAWN_PST, KNIGHT_PST, BISHOP_PST, ROOK_PST, QUEEN_PST, KING_PST
];

pub const KNIGHT_MOBILITY: [S; 9] = [
    s!(-36, -45), s!(-11, -11), s!(3, 4), s!(7, 9), s!(9, 13), s!(9, 16), s!(8, 12), s!(3, 4), s!(0, -7), 
];
pub const BISHOP_MOBILITY: [S; 14] = [
    s!(-34, -80), s!(-21, -49), s!(-9, -30), s!(-3, -11), s!(6, 3), s!(8, 17), s!(13, 22), s!(15, 29), s!(15, 34), s!(17, 30), s!(23, 29), s!(29, 24), s!(20, 40), s!(63, 2), 
];
pub const ROOK_MOBILITY: [S; 15] = [
    s!(-45, -48), s!(-31, -25), s!(-26, -23), s!(-19, -18), s!(-19, -9), s!(-7, -7), s!(-2, -1), s!(7, 1), s!(16, 6), s!(23, 10), s!(30, 14), s!(33, 21), s!(38, 25), s!(44, 24), s!(45, 22), 
];
pub const QUEEN_MOBILITY: [S; 28] = [
    s!(-59, -54), s!(-53, -139), s!(-56, -104), s!(-53, -67), s!(-50, -55), s!(-44, -42), s!(-40, -28), s!(-41, -5), s!(-40, 10), s!(-36, 9), s!(-37, 19), s!(-35, 28), s!(-33, 30), s!(-34, 39), s!(-32, 45), s!(-30, 51), s!(-33, 62), s!(-32, 68), s!(-25, 67), s!(-20, 71), s!(-20, 75), s!(8, 58), s!(20, 54), s!(12, 56), s!(66, 34), s!(201, -44), s!(253, -74), s!(525, -200), 
];

pub const PASSED_PAWNS: [S; 64] = [
    s!(0), s!(0), s!(0), s!(0), s!(0), s!(0), s!(0), s!(0), s!(-7, 24), s!(-8, 22), s!(-5, 21), s!(-7, 19), s!(-8, 21), s!(-8, 22), s!(-6, 21), s!(-6, 23), s!(24, 185), s!(35, 182), s!(24, 157), s!(2, 98), s!(0, 117), s!(2, 141), s!(-55, 153), s!(-74, 186), s!(11, 88), s!(8, 84), s!(25, 63), s!(18, 58), s!(2, 55), s!(14, 63), s!(-35, 86), s!(-34, 94), s!(-6, 44), s!(-22, 35), s!(-29, 32), s!(-19, 23), s!(-24, 26), s!(-14, 30), s!(-41, 50), s!(-12, 45), s!(-19, -3), s!(-30, 6), s!(-37, 12), s!(-38, 1), s!(-28, 0), s!(-16, 4), s!(-27, 22), s!(0, 2), s!(-25, -5), s!(-16, -2), s!(-25, 5), s!(-35, -3), s!(-11, -17), s!(-3, -8), s!(15, -5), s!(-15, 3), s!(0), s!(0), s!(0), s!(0), s!(0), s!(0), s!(0), s!(0), 
];