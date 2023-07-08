use egui::{emath::Rot2, epaint::PathShape, Painter, Pos2, Rect, Stroke};
use hardfiskur_core::board::{Color, Square};

use crate::{
    base_board::BaseBoard,
    constants::{
        ARROW_COLOR, ARROW_HEAD_SIZE, ARROW_SELECTED_HEAD_SIZE, ARROW_SELECTED_WIDTH, ARROW_WIDTH,
        HIGHLIGHTED_CIRCLE_SELECTED_WIDTH, HIGHLIGHTED_CIRCLE_WIDTH, SCALE,
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Arrow {
    pub start: Square,
    pub end: Square,
}

impl Arrow {
    pub fn draw(&self, painter: &Painter, board_rect: Rect, perspective: Color, is_selected: bool) {
        let origin = BaseBoard::square_center(self.start, board_rect, perspective);

        if self.start == self.end {
            self.draw_circle(painter, origin, is_selected);
        } else {
            let end = BaseBoard::square_center(self.end, board_rect, perspective);
            self.draw_arrow(painter, origin, end, is_selected);
        }
    }

    fn draw_circle(&self, painter: &Painter, origin: Pos2, is_selected: bool) {
        let stroke_width = if is_selected {
            HIGHLIGHTED_CIRCLE_SELECTED_WIDTH
        } else {
            HIGHLIGHTED_CIRCLE_WIDTH
        };

        painter.circle_stroke(
            origin,
            SCALE / 2.0 - HIGHLIGHTED_CIRCLE_WIDTH,
            Stroke {
                width: stroke_width,
                color: ARROW_COLOR,
            },
        )
    }

    fn draw_arrow(&self, painter: &Painter, origin: Pos2, end: Pos2, is_selected: bool) {
        use std::f32::consts::PI;
        const SEMICIRCLE_POINTS: usize = 8;

        let (width, head_size) = if is_selected {
            (ARROW_SELECTED_WIDTH, ARROW_SELECTED_HEAD_SIZE)
        } else {
            (ARROW_WIDTH, ARROW_HEAD_SIZE)
        };

        let vector = end - origin;

        let ortho = -vector.normalized().rot90();

        let mut points = Vec::with_capacity(SEMICIRCLE_POINTS + 5);

        for i in 0..=SEMICIRCLE_POINTS {
            let rot = Rot2::from_angle(PI * (1.0 - i as f32 / SEMICIRCLE_POINTS as f32));
            points.push(origin + rot * ortho * width / 2.0);
        }

        let arrowhead_start = end - vector.normalized() * head_size;
        points.push(arrowhead_start + ortho * width / 2.0);
        points.push(arrowhead_start + ortho * head_size * 2.0 / 3.0);
        points.push(end);
        points.push(arrowhead_start - ortho * head_size * 2.0 / 3.0);
        points.push(arrowhead_start - ortho * width / 2.0);

        painter.add(PathShape {
            points,
            closed: true,
            fill: ARROW_COLOR,
            stroke: Stroke::NONE,
        });
    }
}
