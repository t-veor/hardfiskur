use egui::Color32;

pub const CHESS_PIECES_SPRITE: &[u8] = include_bytes!("chess_pieces_sprite.png");

pub const SCALE: f32 = 80.0;

pub const BOARD_WHITE: Color32 = Color32::from_rgb(0xf0, 0xd9, 0xb5);
pub const BOARD_BLACK: Color32 = Color32::from_rgb(0xb5, 0x88, 0x63);
pub const BOARD_PRIMARY: Color32 = Color32::from_rgba_premultiplied(20, 85, 30, 0xb0);
pub const BOARD_LAST_MOVE: Color32 = Color32::from_rgba_premultiplied(0x66, 0x85, 0x00, 0x68);
pub const BOARD_BITBOARD_HIGHLIGHT: Color32 = Color32::from_rgba_premultiplied(192, 64, 64, 192);
