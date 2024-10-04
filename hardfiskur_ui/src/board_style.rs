use egui::{vec2, Color32, Pos2, Rect, Vec2};
use hardfiskur_core::board::{Color, Square};

use crate::constants::{
    BOARD_BITBOARD_HIGHLIGHT, BOARD_BLACK, BOARD_BLACK_FADED, BOARD_LAST_MOVE, BOARD_WHITE,
    BOARD_WHITE_FADED, DEFAULT_BOARD_SIZE,
};

#[derive(Debug, Clone, PartialEq)]
pub struct BoardStyle {
    pub square_size: f32,
}

impl BoardStyle {
    pub fn new(square_size: f32) -> Self {
        Self { square_size }
    }

    pub fn arrow_width(&self) -> f32 {
        12.0 / 80.0 * self.square_size
    }

    pub fn arrow_selected_width(&self) -> f32 {
        10.0 / 80.0 * self.square_size
    }

    pub fn arrow_head_size(&self) -> f32 {
        37.5 / 80.0 * self.square_size
    }

    pub fn arrow_selected_head_size(&self) -> f32 {
        32.0 / 80.0 * self.square_size
    }

    pub fn highlighted_circle_width(&self) -> f32 {
        6.0 / 80.0 * self.square_size
    }

    pub fn highlighted_circle_selected_width(&self) -> f32 {
        4.0 / 80.0 * self.square_size
    }

    pub fn from_board_size(board_size: f32) -> Self {
        Self::new(board_size / 8.0)
    }

    pub fn board_to_render_coords(&self, perspective: Color, coords: Pos2) -> Pos2 {
        let Pos2 { x, y } = coords;
        let (x, y) = if perspective.is_white() {
            (x, 8.0 - y)
        } else {
            (8.0 - x, y)
        };

        Pos2::new(x * self.square_size, y * self.square_size)
    }

    pub fn render_to_board_coords(&self, perspective: Color, coords: Pos2) -> Pos2 {
        let Vec2 { x, y } = coords.to_vec2() / self.square_size;
        if perspective.is_white() {
            Pos2::new(x, 8.0 - y)
        } else {
            Pos2::new(8.0 - x, y)
        }
    }

    pub fn square_center(&self, square: Square, board_rect: Rect, perspective: Color) -> Pos2 {
        board_rect.left_top()
            + self
                .board_to_render_coords(
                    perspective,
                    Pos2::new(square.file() as f32 + 0.5, square.rank() as f32 + 0.5),
                )
                .to_vec2()
    }

    pub fn board_square_centered_at(&self, pos: Pos2) -> Rect {
        Rect::from_center_size(pos, Vec2::splat(self.square_size))
    }

    pub fn board_square(&self, square: Square, board_rect: Rect, perspective: Color) -> Rect {
        self.board_square_centered_at(self.square_center(square, board_rect, perspective))
    }

    pub fn piece_surrounders(
        &self,
        square: Square,
        board_rect: Rect,
        perspective: Color,
    ) -> Vec<Vec<Pos2>> {
        let dst_rect = self.board_square(square, board_rect, perspective);

        vec![
            vec![
                dst_rect.left_top(),
                dst_rect.left_top() + vec2(self.square_size * 0.25, 0.0),
                dst_rect.left_top() + vec2(0.0, self.square_size * 0.25),
            ],
            vec![
                dst_rect.right_top(),
                dst_rect.right_top() + vec2(-self.square_size * 0.25, 0.0),
                dst_rect.right_top() + vec2(0.0, self.square_size * 0.25),
            ],
            vec![
                dst_rect.left_bottom(),
                dst_rect.left_bottom() + vec2(self.square_size * 0.25, 0.0),
                dst_rect.left_bottom() + vec2(0.0, -self.square_size * 0.25),
            ],
            vec![
                dst_rect.right_bottom(),
                dst_rect.right_bottom() + vec2(-self.square_size * 0.25, 0.0),
                dst_rect.right_bottom() + vec2(1.0, -self.square_size * 0.25),
            ],
        ]
    }

    pub fn board_colors(&self, faded: bool) -> (Color32, Color32) {
        if faded {
            (BOARD_WHITE_FADED, BOARD_BLACK_FADED)
        } else {
            (BOARD_WHITE, BOARD_BLACK)
        }
    }

    pub fn last_move_color(&self) -> Color32 {
        BOARD_LAST_MOVE
    }

    pub fn bitboard_highlight_color(&self) -> Color32 {
        BOARD_BITBOARD_HIGHLIGHT
    }
}

impl Default for BoardStyle {
    fn default() -> Self {
        Self::from_board_size(DEFAULT_BOARD_SIZE)
    }
}
