use egui::{
    emath::Rot2,
    epaint::{PathShape, PathStroke},
    Painter, Pos2, Rect, Stroke,
};
use hardfiskur_core::board::{Color, Square};

use crate::{board_style::BoardStyle, constants::ARROW_COLOR};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Arrow {
    pub start: Square,
    pub end: Square,
}

impl Arrow {
    pub fn draw(
        &self,
        painter: &Painter,
        style: &BoardStyle,
        board_rect: Rect,
        perspective: Color,
        is_selected: bool,
    ) {
        let origin = style.square_center(self.start, board_rect, perspective);

        if self.start == self.end {
            self.draw_circle(painter, style, origin, is_selected);
        } else {
            let end = style.square_center(self.end, board_rect, perspective);
            self.draw_arrow(painter, style, origin, end, is_selected);
        }
    }

    fn draw_circle(&self, painter: &Painter, style: &BoardStyle, origin: Pos2, is_selected: bool) {
        let stroke_width = if is_selected {
            style.highlighted_circle_selected_width()
        } else {
            style.highlighted_circle_width()
        };

        painter.circle_stroke(
            origin,
            style.square_size / 2.0 - style.highlighted_circle_selected_width(),
            Stroke {
                width: stroke_width,
                color: ARROW_COLOR,
            },
        );
    }

    fn draw_arrow(
        &self,
        painter: &Painter,
        style: &BoardStyle,
        origin: Pos2,
        end: Pos2,
        is_selected: bool,
    ) {
        use std::f32::consts::PI;
        const SEMICIRCLE_POINTS: usize = 8;

        let (width, head_size) = if is_selected {
            (
                style.arrow_selected_width(),
                style.arrow_selected_head_size(),
            )
        } else {
            (style.arrow_width(), style.arrow_head_size())
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
            stroke: PathStroke::NONE,
        });
    }
}
