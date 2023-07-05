use egui::{
    epaint::ColorImage, Align2, Painter, PointerButton, Pos2, Rect, Sense, TextureHandle, Ui, Vec2,
};
use hardfiskur_core::board::{Bitboard, Color, Move, Piece, PieceType, Square};

use crate::constants::{
    BOARD_BITBOARD_HIGHLIGHT, BOARD_BLACK, BOARD_WHITE, CHESS_PIECES_SPRITE, SCALE,
};

#[derive(Default)]
pub struct ChessUIData<'a> {
    pub pieces: &'a [Option<Piece>],
    pub possible_moves: &'a [Move],
    pub perspective: Color,
    pub display_bitboard: Option<Bitboard>,
    pub drag_mask: Bitboard,
}

impl<'a> ChessUIData<'a> {
    fn piece_at(&self, square: Square) -> Option<Piece> {
        self.pieces.get(square.index()).copied().flatten()
    }
}

#[derive(Debug)]
pub struct ChessUIResponse {
    pub egui_response: egui::Response,
    pub dropped: Option<(Square, Square)>,
    pub clicked_square: Option<Square>,
}

#[derive(Default)]
pub struct ChessUI {
    piece_sprites: Option<TextureHandle>,
    last_mouse_pos: Option<Pos2>,
    drag_start: Option<Square>,
}

impl ChessUI {
    pub fn ui(&mut self, ui: &mut Ui, data: ChessUIData<'_>) -> ChessUIResponse {
        let board_size = Vec2::splat(SCALE * 8.0);
        let (egui_response, painter) = ui.allocate_painter(board_size, Sense::click_and_drag());

        self.last_mouse_pos = egui_response.interact_pointer_pos().or(self.last_mouse_pos);
        let board_rect = Rect::from_center_size(egui_response.rect.center(), board_size);
        let rel_mouse_pos = self
            .last_mouse_pos
            .map(|pos| pos - board_rect.left_top())
            .unwrap_or(Vec2::ZERO);

        self.paint_board(&data, &painter, board_rect);

        self.paint_bitboard(&data, ui, &painter, board_rect);

        self.paint_pieces(&data, ui, board_rect, rel_mouse_pos);

        let mut response = ChessUIResponse {
            egui_response,
            dropped: None,
            clicked_square: None,
        };

        self.handle_mouse(&data, &mut response, board_rect, rel_mouse_pos);

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

    fn board_to_render_coords(perspective: Color, coords: Vec2) -> Vec2 {
        let Vec2 { x, y } = coords;
        let (x, y) = if perspective.is_white() {
            (x, 8.0 - y)
        } else {
            (8.0 - x, y)
        };

        Vec2::new(x * SCALE, y * SCALE)
    }

    fn render_to_board_coords(perspective: Color, coords: Vec2) -> Vec2 {
        let Vec2 { x, y } = coords / SCALE;
        if perspective.is_white() {
            Vec2::new(x, 8.0 - y)
        } else {
            Vec2::new(8.0 - x, y)
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

    fn dst_rect(rank: u8, file: u8, board_rect: Rect, perspective: Color) -> Rect {
        Rect::from_center_size(
            board_rect.left_top()
                + Self::board_to_render_coords(
                    perspective,
                    Vec2::new(file as f32 + 0.5, rank as f32 + 0.5),
                ),
            Vec2::splat(SCALE),
        )
    }

    fn handle_mouse(
        &mut self,
        data: &ChessUIData<'_>,
        response: &mut ChessUIResponse,
        board_rect: Rect,
        rel_mouse_pos: Vec2,
    ) {
        let mouse_square = {
            let Vec2 { x, y } = Self::render_to_board_coords(data.perspective, rel_mouse_pos);
            if (0.0..8.0).contains(&x) && (0.0..8.0).contains(&y) {
                Some(Square::new_unchecked(
                    y.max(0.0).min(7.0) as _,
                    x.max(0.0).min(7.0) as _,
                ))
            } else {
                None
            }
        };

        if response
            .egui_response
            .drag_started_by(PointerButton::Primary)
        {
            if let Some(start) = mouse_square {
                if data.drag_mask.get(start) && data.piece_at(start).is_some() {
                    self.drag_start = mouse_square;
                }
            }
        }

        if response
            .egui_response
            .drag_released_by(PointerButton::Primary)
        {
            if let (Some(start), Some(end)) = (self.drag_start, mouse_square) {
                if data.piece_at(start).is_some() {
                    response.dropped = Some((start, end));
                }
            }
            self.drag_start = None;
        }

        if response.egui_response.clicked() {
            response.clicked_square = mouse_square;
        }
    }

    fn paint_board(&mut self, data: &ChessUIData<'_>, painter: &Painter, board_rect: Rect) {
        let colors = [BOARD_BLACK, BOARD_WHITE];

        for rank in 0..8 {
            for file in 0..8 {
                let color = colors[(rank + file) % 2];
                let opposite_color = colors[(rank + file + 1) % 2];

                let rect = Self::dst_rect(rank as _, file as _, board_rect, data.perspective);

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
    }

    fn paint_pieces(
        &mut self,
        data: &ChessUIData<'_>,
        ui: &mut Ui,
        board_rect: Rect,
        rel_mouse_pos: Vec2,
    ) {
        let sprite_handle = self.get_piece_sprite(ui.ctx());

        for rank in 0..8 {
            for file in 0..8 {
                let square = Square::new(rank, file).unwrap();

                if self.drag_start == Some(square) {
                    continue;
                }

                if let Some(piece) = data.pieces.get(square.index()).copied().flatten() {
                    let src_rect = Self::get_piece_uv(piece);
                    let dst_rect = Self::dst_rect(rank, file, board_rect, data.perspective);

                    egui::Image::new(&sprite_handle, Vec2::splat(SCALE))
                        .uv(src_rect)
                        .paint_at(ui, dst_rect)
                }
            }
        }

        if let Some(drag_from_square) = self.drag_start {
            if let Some(dragged_piece) =
                data.pieces.get(drag_from_square.index()).copied().flatten()
            {
                let src_rect = Self::get_piece_uv(dragged_piece);
                let dst_rect = Rect::from_center_size(
                    board_rect.left_top() + rel_mouse_pos,
                    Vec2::splat(SCALE),
                );

                egui::Image::new(&sprite_handle, Vec2::new(SCALE, SCALE))
                    .uv(src_rect)
                    .paint_at(ui, dst_rect);
            } else {
                // Dragged piece was removed from underneath us...
                // TODO: is this actually right?
            }
        }
    }

    fn paint_bitboard(
        &mut self,
        data: &ChessUIData<'_>,
        ui: &mut Ui,
        painter: &Painter,
        board_rect: Rect,
    ) {
        if let Some(bitboard) = data.display_bitboard {
            for rank in 0..8 {
                for file in 0..8 {
                    let square = Square::new(rank, file).unwrap();
                    if bitboard.get(square) {
                        let rect = Self::dst_rect(rank, file, board_rect, data.perspective);
                        painter.rect_filled(rect, 0.0, BOARD_BITBOARD_HIGHLIGHT);
                    }
                }
            }
        }
    }
}
