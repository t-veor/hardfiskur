use egui::{Painter, PointerButton, Pos2, Rect, Rgba, TextureHandle, Ui, Vec2};
use hardfiskur_core::board::{Color, PieceType, Square};

use crate::constants::SCALE;

use super::BaseBoard;

#[derive(Debug)]
pub struct PromotionUi {
    board_rect: Rect,
    anchor: Pos2,
    for_player: Color,
    direction: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromotionResult {
    ToQueen,
    ToKnight,
    ToRook,
    ToBishop,
    Cancel,
}

impl PromotionResult {
    pub fn into_piece_type(self) -> Option<PieceType> {
        match self {
            PromotionResult::ToQueen => Some(PieceType::Queen),
            PromotionResult::ToKnight => Some(PieceType::Knight),
            PromotionResult::ToRook => Some(PieceType::Rook),
            PromotionResult::ToBishop => Some(PieceType::Bishop),
            PromotionResult::Cancel => None,
        }
    }
}

impl PromotionUi {
    pub fn new(
        promotion_square: Square,
        for_player: Color,
        board_rect: Rect,
        perspective: Color,
    ) -> Self {
        let anchor = BaseBoard::square_center(promotion_square, board_rect, perspective);
        let direction = if anchor.y - board_rect.left_top().y < SCALE * 4.5 {
            SCALE
        } else {
            -SCALE
        };

        Self {
            board_rect,
            anchor,
            for_player,
            direction,
        }
    }

    pub(super) fn handle_input(&self, response: &egui::Response) -> Option<PromotionResult> {
        if !response.clicked_by(PointerButton::Primary) {
            return None;
        }

        let mouse_pos = response.interact_pointer_pos()?;

        if !(self.anchor.x - SCALE / 2.0..=self.anchor.x + SCALE / 2.0).contains(&mouse_pos.x) {
            return Some(PromotionResult::Cancel);
        }

        let item_pos = (mouse_pos.y - self.anchor.y) / self.direction;

        match (item_pos + 0.5).floor() as i32 {
            0 => Some(PromotionResult::ToQueen),
            1 => Some(PromotionResult::ToKnight),
            2 => Some(PromotionResult::ToRook),
            3 => Some(PromotionResult::ToBishop),
            _ => Some(PromotionResult::Cancel),
        }
    }

    pub(super) fn draw(&self, ui: &mut Ui, painter: &Painter, sprite_handle: &TextureHandle) {
        painter.rect_filled(
            self.board_rect,
            0.0,
            Rgba::from_rgba_unmultiplied(0.0, 0.0, 0.0, 0.25),
        );

        let bg_center = self.anchor + Vec2::new(0.0, 1.5 * self.direction);
        let bg_rect = Rect::from_center_size(bg_center, Vec2::new(SCALE, 4.0 * SCALE));

        painter.rect_filled(bg_rect, 8.0, Rgba::from_rgb(0.8, 0.8, 0.8));

        for (i, piece_type) in [
            PieceType::Queen,
            PieceType::Knight,
            PieceType::Rook,
            PieceType::Bishop,
        ]
        .into_iter()
        .enumerate()
        {
            let src_rect = BaseBoard::get_piece_uv(piece_type.with_color(self.for_player));
            let dst_rect_center = self.anchor + Vec2::new(0.0, i as f32 * self.direction);
            let dst_rect = Rect::from_center_size(dst_rect_center, Vec2::splat(SCALE));

            egui::Image::new(sprite_handle)
                .uv(src_rect)
                .paint_at(ui, dst_rect);
        }
    }
}
