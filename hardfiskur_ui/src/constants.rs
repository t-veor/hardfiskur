use egui::Color32;

pub const DEFAULT_BOARD_SIZE: f32 = 640.0;
pub const MIN_BOARD_SIZE: f32 = 80.0;

pub const CHESS_PIECES_SPRITE: &[u8] = include_bytes!("chess_pieces_sprite.png");

pub const BOARD_WHITE: Color32 = Color32::from_rgb(0xf0, 0xd9, 0xb5);
pub const BOARD_BLACK: Color32 = Color32::from_rgb(0xb5, 0x88, 0x63);
pub const BOARD_WHITE_FADED: Color32 = Color32::from_rgb(0xf0, 0xea, 0xe1);
pub const BOARD_BLACK_FADED: Color32 = Color32::from_rgb(0xb5, 0x98, 0x8f);
pub const BOARD_PRIMARY: Color32 = Color32::from_rgba_premultiplied(20, 85, 30, 0xb0);
pub const BOARD_LAST_MOVE: Color32 = Color32::from_rgba_premultiplied(0x33, 0x42, 0x00, 0x34);
pub const BOARD_BITBOARD_HIGHLIGHT: Color32 = Color32::from_rgba_premultiplied(192, 64, 64, 192);

pub const MOVE_COLOR: Color32 = Color32::from_rgba_premultiplied(13, 72, 16, 154);

pub const ARROW_COLOR: Color32 = Color32::from_rgba_premultiplied(13, 72, 16, 154);
