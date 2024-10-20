#![cfg_attr(any(), rustfmt::skip)]

use crate::s;
use super::packed_score::S;

pub const MATERIAL: [S; 6] = [
    s!(108, 190), s!(420, 437), s!(461, 451), s!(593, 804), s!(1243, 1431), s!(0), 
];

pub const PAWN_PST: [S; 64] = [
    s!(0), s!(0), s!(0), s!(0), s!(0), s!(0), s!(0), s!(0), s!(64, 220), s!(85, 210), s!(62, 204), s!(93, 141), s!(85, 131), s!(63, 152), s!(-27, 215), s!(-65, 229), s!(-30, 126), s!(-6, 134), s!(29, 93), s!(34, 63), s!(43, 49), s!(73, 30), s!(45, 91), s!(-9, 90), s!(-39, 26), s!(-10, 10), s!(-8, -16), s!(-5, -28), s!(19, -41), s!(15, -36), s!(18, -11), s!(-10, -10), s!(-51, -10), s!(-14, -14), s!(-14, -39), s!(7, -44), s!(9, -46), s!(2, -45), s!(9, -29), s!(-31, -36), s!(-42, -20), s!(-18, -18), s!(-14, -41), s!(-7, -26), s!(14, -36), s!(-2, -40), s!(32, -32), s!(-6, -44), s!(-39, -14), s!(-10, -14), s!(-21, -30), s!(-14, -17), s!(3, -18), s!(25, -33), s!(52, -35), s!(-11, -45), s!(0), s!(0), s!(0), s!(0), s!(0), s!(0), s!(0), s!(0), 
];
pub const KNIGHT_PST: [S; 64] = [
    s!(-218, -66), s!(-153, -15), s!(-73, 5), s!(-40, -1), s!(24, -7), s!(-87, -26), s!(-138, -3), s!(-134, -111), s!(-44, -16), s!(-9, 9), s!(55, 7), s!(54, 16), s!(54, -2), s!(120, -16), s!(6, -4), s!(24, -41), s!(-4, -1), s!(44, 16), s!(71, 53), s!(91, 49), s!(141, 27), s!(150, 21), s!(83, 2), s!(40, -14), s!(5, 13), s!(20, 41), s!(49, 65), s!(83, 66), s!(53, 67), s!(89, 60), s!(29, 41), s!(52, 2), s!(-11, 14), s!(5, 27), s!(29, 66), s!(34, 65), s!(47, 69), s!(40, 52), s!(33, 27), s!(-2, 9), s!(-36, -7), s!(-8, 17), s!(16, 31), s!(22, 57), s!(38, 53), s!(25, 25), s!(20, 10), s!(-12, -5), s!(-45, -17), s!(-27, 0), s!(-12, 13), s!(14, 16), s!(13, 15), s!(13, 10), s!(-6, -9), s!(-13, -2), s!(-101, -22), s!(-18, -25), s!(-40, -5), s!(-15, 2), s!(-10, 1), s!(-5, -10), s!(-15, -22), s!(-67, -28), 
];
pub const BISHOP_PST: [S; 64] = [
    s!(-12, 11), s!(-62, 23), s!(-44, 18), s!(-98, 37), s!(-98, 31), s!(-54, 15), s!(-17, 5), s!(-60, 13), s!(-14, -1), s!(24, 10), s!(6, 17), s!(-7, 16), s!(26, 9), s!(34, 2), s!(19, 17), s!(19, -13), s!(3, 28), s!(37, 17), s!(49, 23), s!(62, 14), s!(52, 18), s!(88, 24), s!(67, 13), s!(52, 17), s!(-3, 23), s!(17, 35), s!(41, 27), s!(58, 44), s!(53, 35), s!(42, 31), s!(22, 26), s!(5, 19), s!(-1, 14), s!(12, 29), s!(18, 38), s!(51, 34), s!(47, 34), s!(20, 32), s!(14, 24), s!(13, 4), s!(18, 15), s!(21, 26), s!(30, 30), s!(26, 31), s!(32, 37), s!(33, 28), s!(24, 17), s!(35, 5), s!(18, 14), s!(34, 7), s!(29, 2), s!(17, 22), s!(30, 20), s!(40, 9), s!(56, 14), s!(25, -4), s!(9, 2), s!(30, 13), s!(24, 8), s!(5, 18), s!(12, 15), s!(6, 26), s!(34, -7), s!(19, -10), 
];
pub const ROOK_PST: [S; 64] = [
    s!(41, 55), s!(29, 62), s!(34, 71), s!(38, 65), s!(61, 55), s!(95, 42), s!(79, 44), s!(96, 39), s!(14, 57), s!(8, 75), s!(42, 75), s!(73, 60), s!(54, 60), s!(93, 45), s!(88, 39), s!(119, 21), s!(-9, 57), s!(26, 58), s!(20, 60), s!(28, 56), s!(66, 39), s!(78, 32), s!(141, 19), s!(104, 15), s!(-22, 58), s!(-5, 55), s!(-2, 65), s!(5, 60), s!(10, 44), s!(21, 38), s!(41, 32), s!(42, 25), s!(-35, 48), s!(-37, 54), s!(-24, 56), s!(-12, 54), s!(-8, 48), s!(-19, 46), s!(17, 29), s!(-2, 24), s!(-35, 41), s!(-28, 41), s!(-16, 40), s!(-9, 42), s!(-1, 37), s!(1, 26), s!(40, 3), s!(11, 5), s!(-36, 34), s!(-22, 38), s!(-1, 39), s!(0, 40), s!(9, 29), s!(15, 22), s!(34, 11), s!(-12, 21), s!(-5, 33), s!(-3, 37), s!(14, 41), s!(24, 36), s!(30, 27), s!(24, 27), s!(27, 20), s!(6, 10), 
];
pub const QUEEN_PST: [S; 64] = [
    s!(1, 142), s!(-20, 179), s!(22, 193), s!(64, 179), s!(72, 172), s!(79, 171), s!(107, 106), s!(38, 158), s!(20, 124), s!(-4, 160), s!(0, 209), s!(-15, 240), s!(-8, 261), s!(60, 208), s!(37, 183), s!(111, 142), s!(26, 125), s!(21, 145), s!(29, 186), s!(41, 198), s!(54, 222), s!(110, 190), s!(125, 142), s!(119, 136), s!(8, 136), s!(15, 159), s!(16, 174), s!(16, 203), s!(21, 222), s!(36, 214), s!(41, 200), s!(55, 175), s!(17, 132), s!(13, 163), s!(15, 168), s!(24, 196), s!(27, 187), s!(26, 184), s!(38, 172), s!(42, 160), s!(18, 114), s!(31, 126), s!(27, 155), s!(28, 148), s!(31, 159), s!(42, 152), s!(50, 141), s!(42, 126), s!(25, 110), s!(31, 112), s!(43, 107), s!(50, 112), s!(48, 119), s!(62, 84), s!(67, 53), s!(79, 32), s!(30, 101), s!(29, 99), s!(42, 98), s!(55, 107), s!(48, 97), s!(27, 99), s!(45, 73), s!(42, 64), 
];
pub const KING_PST: [S; 64] = [
    s!(67, -127), s!(29, -56), s!(79, -48), s!(-60, 2), s!(1, -12), s!(30, -4), s!(54, -10), s!(146, -116), s!(-90, -9), s!(-7, 27), s!(-58, 39), s!(38, 23), s!(-6, 43), s!(15, 60), s!(-4, 57), s!(-41, 20), s!(-115, 11), s!(41, 35), s!(-66, 60), s!(-75, 69), s!(-55, 73), s!(45, 66), s!(21, 67), s!(-55, 32), s!(-93, -3), s!(-101, 44), s!(-111, 67), s!(-170, 84), s!(-167, 84), s!(-122, 79), s!(-114, 67), s!(-162, 37), s!(-90, -18), s!(-97, 25), s!(-138, 59), s!(-188, 80), s!(-188, 81), s!(-146, 66), s!(-142, 48), s!(-184, 26), s!(-47, -27), s!(-27, 6), s!(-108, 37), s!(-128, 54), s!(-121, 55), s!(-115, 43), s!(-57, 18), s!(-80, 0), s!(59, -51), s!(7, -14), s!(-17, 6), s!(-66, 22), s!(-67, 25), s!(-39, 13), s!(26, -14), s!(35, -39), s!(46, -95), s!(94, -74), s!(67, -46), s!(-72, -19), s!(15, -48), s!(-35, -23), s!(67, -62), s!(61, -102), 
];

pub const PIECE_SQUARE_TABLES: [[S; 64]; 6] = [
    PAWN_PST, KNIGHT_PST, BISHOP_PST, ROOK_PST, QUEEN_PST, KING_PST
];

pub const KNIGHT_MOBILITY: [S; 9] = [
    s!(-36, -39), s!(-11, -9), s!(3, 3), s!(7, 8), s!(10, 10), s!(10, 13), s!(9, 8), s!(3, 0), s!(1, -11), 
];
pub const BISHOP_MOBILITY: [S; 14] = [
    s!(-35, -76), s!(-22, -45), s!(-10, -27), s!(-3, -9), s!(6, 3), s!(8, 17), s!(13, 22), s!(15, 27), s!(15, 32), s!(18, 27), s!(24, 26), s!(30, 20), s!(22, 32), s!(70, -6), 
];
pub const ROOK_MOBILITY: [S; 15] = [
    s!(-46, -47), s!(-32, -25), s!(-27, -22), s!(-20, -17), s!(-19, -9), s!(-8, -7), s!(-2, -1), s!(7, 1), s!(16, 6), s!(24, 9), s!(32, 11), s!(35, 17), s!(40, 22), s!(47, 19), s!(46, 19), 
];
pub const QUEEN_MOBILITY: [S; 28] = [
    s!(-52, -54), s!(-46, -139), s!(-49, -104), s!(-46, -67), s!(-43, -55), s!(-37, -43), s!(-34, -30), s!(-34, -7), s!(-33, 8), s!(-29, 6), s!(-29, 16), s!(-28, 25), s!(-25, 26), s!(-26, 35), s!(-25, 40), s!(-23, 46), s!(-25, 56), s!(-24, 61), s!(-17, 59), s!(-12, 63), s!(-11, 66), s!(17, 49), s!(29, 46), s!(23, 45), s!(76, 24), s!(207, -52), s!(232, -69), s!(353, -114), 
];

pub const PASSED_PAWNS: [S; 64] = [s!(0); 64];