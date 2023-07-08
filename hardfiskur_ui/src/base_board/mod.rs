use egui::{
    epaint::ColorImage, Align2, Painter, PointerButton, Pos2, Rect, Sense, TextureHandle, Ui, Vec2,
};
use hardfiskur_core::board::{Bitboard, Color, Move, Piece, PieceType, Square};

use crate::constants::{
    BOARD_BITBOARD_HIGHLIGHT, BOARD_BLACK, BOARD_WHITE, CHESS_PIECES_SPRITE, SCALE,
};

use self::arrow::Arrow;

mod arrow;

#[derive(Default)]
pub struct BaseBoardData<'a> {
    pub pieces: &'a [Option<Piece>],
    pub possible_moves: &'a [Move],
    pub perspective: Color,
    pub display_bitboard: Option<Bitboard>,
    pub drag_mask: Bitboard,
    pub allow_arrows: bool,
}

impl<'a> BaseBoardData<'a> {
    fn piece_at(&self, square: Square) -> Option<Piece> {
        self.pieces.get(square.index()).copied().flatten()
    }
}

#[derive(Debug)]
pub struct BaseBoardResponse {
    pub egui_response: egui::Response,
    pub dropped: Option<(Square, Square)>,
    pub clicked_square: Option<Square>,
}

pub struct BaseBoard {
    piece_sprites: Option<TextureHandle>,

    board_rect: Rect,
    rel_mouse_pos: Pos2,
    mouse_square: Option<Square>,

    drag_start: Option<Square>,

    arrow_start: Option<Square>,
    arrows: Vec<Arrow>,
}

impl Default for BaseBoard {
    fn default() -> Self {
        Self {
            piece_sprites: None,
            board_rect: Rect::from_min_max(Pos2::ZERO, Pos2::ZERO),
            rel_mouse_pos: Pos2::ZERO,
            mouse_square: None,
            drag_start: None,
            arrow_start: None,
            arrows: Vec::new(),
        }
    }
}

impl BaseBoard {
    pub fn ui(&mut self, ui: &mut Ui, data: BaseBoardData<'_>) -> BaseBoardResponse {
        let board_size = Vec2::splat(SCALE * 8.0);
        let (egui_response, painter) = ui.allocate_painter(board_size, Sense::click_and_drag());

        self.board_rect = Rect::from_center_size(egui_response.rect.center(), board_size);
        if let Some(mouse_pos) = egui_response.interact_pointer_pos() {
            self.rel_mouse_pos = (mouse_pos - self.board_rect.left_top()).to_pos2();
        }
        self.mouse_square = {
            let Pos2 { x, y } = Self::render_to_board_coords(data.perspective, self.rel_mouse_pos);
            if (0.0..8.0).contains(&x) && (0.0..8.0).contains(&y) {
                Some(Square::new_unchecked(
                    y.max(0.0).min(7.0) as _,
                    x.max(0.0).min(7.0) as _,
                ))
            } else {
                None
            }
        };

        self.paint_board(&painter, &data);

        self.paint_bitboard(&painter, &data);

        self.paint_pieces(ui, &data);

        self.paint_arrows(&painter, &data);

        let mut response = BaseBoardResponse {
            egui_response,
            dropped: None,
            clicked_square: None,
        };

        self.handle_input(ui, &data, &mut response);

        response
    }

    fn get_piece_sprite(&mut self, ctx: &egui::Context) -> TextureHandle {
        self.piece_sprites
            .get_or_insert_with(|| {
                let image = image::load_from_memory(CHESS_PIECES_SPRITE)
                    .expect("Couldn't load chess pieces sprite");
                let size = [image.width() as _, image.height() as _];
                let image_buffer = image.to_rgba8();
                let pixels = image_buffer.as_flat_samples();
                let image = ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
                ctx.load_texture("chess-sprites", image, Default::default())
            })
            .clone()
    }

    fn board_to_render_coords(perspective: Color, coords: Pos2) -> Pos2 {
        let Pos2 { x, y } = coords;
        let (x, y) = if perspective.is_white() {
            (x, 8.0 - y)
        } else {
            (8.0 - x, y)
        };

        Pos2::new(x * SCALE, y * SCALE)
    }

    fn render_to_board_coords(perspective: Color, coords: Pos2) -> Pos2 {
        let Vec2 { x, y } = coords.to_vec2() / SCALE;
        if perspective.is_white() {
            Pos2::new(x, 8.0 - y)
        } else {
            Pos2::new(8.0 - x, y)
        }
    }

    fn get_piece_uv(piece: Piece) -> Rect {
        let x = match piece.piece_type() {
            PieceType::King => 0.0,
            PieceType::Queen => 1.0,
            PieceType::Bishop => 2.0,
            PieceType::Knight => 3.0,
            PieceType::Rook => 4.0,
            PieceType::Pawn => 5.0,
        } / 6.0;
        let y = match piece.color() {
            Color::White => 0.0,
            Color::Black => 1.0,
        } / 2.0;

        Rect::from_min_size(Pos2::new(x, y), Vec2::new(1.0 / 6.0, 0.5))
    }

    fn square_center(square: Square, board_rect: Rect, perspective: Color) -> Pos2 {
        board_rect.left_top()
            + Self::board_to_render_coords(
                perspective,
                Pos2::new(square.file() as f32 + 0.5, square.rank() as f32 + 0.5),
            )
            .to_vec2()
    }

    fn dst_rect(square: Square, board_rect: Rect, perspective: Color) -> Rect {
        Rect::from_center_size(
            Self::square_center(square, board_rect, perspective),
            Vec2::splat(SCALE),
        )
    }

    fn handle_input(
        &mut self,
        ui: &mut Ui,
        data: &BaseBoardData<'_>,
        response: &mut BaseBoardResponse,
    ) {
        self.handle_drag_piece(data, response);
        self.handle_draw_arrows(ui, data, response);
        self.handle_clicks(response);
    }

    fn handle_drag_piece(&mut self, data: &BaseBoardData<'_>, response: &mut BaseBoardResponse) {
        if response
            .egui_response
            .drag_started_by(PointerButton::Primary)
        {
            if let Some(start) = self.mouse_square {
                if data.drag_mask.get(start) && data.piece_at(start).is_some() {
                    self.drag_start = self.mouse_square;
                }
            }
        }

        if response
            .egui_response
            .drag_released_by(PointerButton::Primary)
        {
            if let (Some(start), Some(end)) = (self.drag_start, self.mouse_square) {
                if data.piece_at(start).is_some() {
                    response.dropped = Some((start, end));
                }
            }
            self.drag_start = None;
        }
    }

    fn handle_draw_arrows(
        &mut self,
        ui: &mut Ui,
        data: &BaseBoardData<'_>,
        response: &mut BaseBoardResponse,
    ) {
        if data.allow_arrows
            && response
                .egui_response
                .drag_started_by(PointerButton::Secondary)
        {
            self.arrow_start = self.mouse_square;
        }

        if response
            .egui_response
            .drag_released_by(PointerButton::Secondary)
        {
            if let Some(start) = self.arrow_start.take() {
                let end = self.mouse_square.unwrap_or(start);
                let arrow = Arrow { start, end };
                if let Some(idx) = self.arrows.iter().position(|a| a == &arrow) {
                    self.arrows.swap_remove(idx);
                } else {
                    self.arrows.push(arrow)
                }
            }
        }

        if ui.input(|i| i.pointer.button_released(PointerButton::Primary)) {
            self.arrows.clear();
        }
    }

    fn handle_clicks(&mut self, response: &mut BaseBoardResponse) {
        if response.egui_response.clicked_by(PointerButton::Primary) {
            response.clicked_square = self.mouse_square;
        }
    }

    fn paint_board(&mut self, painter: &Painter, data: &BaseBoardData<'_>) {
        for square in Square::all() {
            let (rank, file) = (square.rank(), square.file());
            let (color, opposite_color) = if (rank + file) % 2 > 0 {
                (BOARD_WHITE, BOARD_BLACK)
            } else {
                (BOARD_BLACK, BOARD_WHITE)
            };

            let rect = Self::dst_rect(square, self.board_rect, data.perspective);

            painter.rect_filled(rect, 0.0, color);

            // Draw coordinate indicators
            let (is_visually_last_row, is_visually_last_column) = match data.perspective {
                Color::White => (rank == 0, file == 7),
                Color::Black => (rank == 7, file == 0),
            };

            if is_visually_last_row {
                painter.text(
                    rect.left_bottom() + Vec2::new(2.0, -2.0),
                    Align2::LEFT_BOTTOM,
                    (b'a' + file as u8) as char,
                    Default::default(),
                    opposite_color,
                );
            }

            if is_visually_last_column {
                painter.text(
                    rect.right_top() + Vec2::new(-2.0, 2.0),
                    Align2::RIGHT_TOP,
                    (b'1' + rank as u8) as char,
                    Default::default(),
                    opposite_color,
                );
            }
        }
    }

    fn paint_pieces(&mut self, ui: &mut Ui, data: &BaseBoardData<'_>) {
        let sprite_handle = self.get_piece_sprite(ui.ctx());

        for square in Square::all() {
            if self.drag_start == Some(square) {
                continue;
            }

            if let Some(piece) = data.pieces.get(square.index()).copied().flatten() {
                let src_rect = Self::get_piece_uv(piece);
                let dst_rect = Self::dst_rect(square, self.board_rect, data.perspective);

                egui::Image::new(&sprite_handle, Vec2::splat(SCALE))
                    .uv(src_rect)
                    .paint_at(ui, dst_rect)
            }
        }

        if let Some(drag_from_square) = self.drag_start {
            if let Some(dragged_piece) =
                data.pieces.get(drag_from_square.index()).copied().flatten()
            {
                let src_rect = Self::get_piece_uv(dragged_piece);
                let dst_rect = Rect::from_center_size(
                    self.board_rect.left_top() + self.rel_mouse_pos.to_vec2(),
                    Vec2::splat(SCALE),
                );

                egui::Image::new(&sprite_handle, Vec2::new(SCALE, SCALE))
                    .uv(src_rect)
                    .paint_at(ui, dst_rect);
            } else {
                // Dragged piece was removed from underneath us...
            }
        }
    }

    fn paint_bitboard(&mut self, painter: &Painter, data: &BaseBoardData<'_>) {
        if let Some(bitboard) = data.display_bitboard {
            for square in Square::all() {
                if bitboard.get(square) {
                    let rect = Self::dst_rect(square, self.board_rect, data.perspective);
                    painter.rect_filled(rect, 0.0, BOARD_BITBOARD_HIGHLIGHT);
                }
            }
        }
    }

    fn paint_arrows(&mut self, painter: &Painter, data: &BaseBoardData<'_>) {
        for arrow in self.arrows.iter() {
            arrow.draw(painter, self.board_rect, data.perspective, false);
        }

        if let Some(start) = self.arrow_start {
            let end = match self.mouse_square {
                Some(end) => end,
                None => start,
            };
            Arrow { start, end }.draw(painter, self.board_rect, data.perspective, true);
        }
    }
}
