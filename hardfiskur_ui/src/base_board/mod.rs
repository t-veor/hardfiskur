use egui::{
    epaint::{ColorImage, PathShape, PathStroke, RectShape},
    vec2, Align2, Color32, Id, Painter, PointerButton, Pos2, Rect, Rgba, Sense, Shadow,
    TextureHandle, Ui, Vec2,
};
use hardfiskur_core::board::{Bitboard, Color, Piece, PieceType, Square};
use sprite_state::{AnimatedPieceState, SpriteState};

use crate::constants::{
    BOARD_BITBOARD_HIGHLIGHT, BOARD_BLACK, BOARD_BLACK_FADED, BOARD_LAST_MOVE, BOARD_WHITE,
    BOARD_WHITE_FADED, CHESS_PIECES_SPRITE, MOVE_COLOR, SCALE,
};

use self::{arrow::Arrow, promo_ui::PromotionUi};
pub use promo_ui::PromotionResult;

mod arrow;
mod promo_ui;
mod sprite_state;

#[derive(Debug)]
pub struct BaseBoardUIProps<'a> {
    pieces: &'a [Option<Piece>],
    possible_moves: &'a [(Square, Square)],
    perspective: Color,
    display_bitboard: Bitboard,
    drag_mask: Bitboard,
    allow_arrows: bool,
    handle_promo_on: Option<(Square, Color)>,
    checked_king_position: Option<Square>,
    fade_out_board: bool,
    show_last_move: Option<(Square, Square)>,
}

impl<'a> BaseBoardUIProps<'a> {
    pub fn new() -> Self {
        Self {
            pieces: &[],
            possible_moves: &[],
            perspective: Color::White,
            display_bitboard: Bitboard::EMPTY,
            drag_mask: Bitboard::ALL,
            allow_arrows: true,
            handle_promo_on: None,
            checked_king_position: None,
            fade_out_board: false,
            show_last_move: None,
        }
    }

    pub fn pieces(mut self, pieces: &'a [Option<Piece>]) -> Self {
        self.pieces = pieces;
        self
    }

    pub fn possible_moves(mut self, moves: &'a [(Square, Square)]) -> Self {
        self.possible_moves = moves;
        self
    }

    pub fn perspective(mut self, perspective: Color) -> Self {
        self.perspective = perspective;
        self
    }

    pub fn display_bitboard(mut self, display_bitboard: Bitboard) -> Self {
        self.display_bitboard = display_bitboard;
        self
    }

    pub fn drag_mask(mut self, drag_mask: Bitboard) -> Self {
        self.drag_mask = drag_mask;
        self
    }

    pub fn allow_arrows(mut self, allow_arrows: bool) -> Self {
        self.allow_arrows = allow_arrows;
        self
    }

    pub fn handle_promo_on(mut self, square: Square, color: Color) -> Self {
        self.handle_promo_on = Some((square, color));
        self
    }

    pub fn checked_king_position(mut self, square: Option<Square>) -> Self {
        self.checked_king_position = square;
        self
    }

    pub fn fade_out_board(mut self, fade_out_board: bool) -> Self {
        self.fade_out_board = fade_out_board;
        self
    }

    pub fn show_last_move(mut self, from: Square, to: Square) -> Self {
        self.show_last_move = Some((from, to));
        self
    }

    fn piece_at(&self, square: Square) -> Option<Piece> {
        self.pieces.get(square.index()).copied().flatten()
    }
}

#[derive(Debug)]
pub struct BaseBoardUIResponse {
    pub egui_response: egui::Response,
    pub holding: Option<Square>,
    pub dropped: Option<(Square, Square)>,
    pub clicked_square: Option<Square>,
    pub promotion_result: Option<PromotionResult>,
}

impl BaseBoardUIResponse {
    pub fn new(egui_response: egui::Response) -> Self {
        Self {
            egui_response,
            holding: None,
            dropped: None,
            clicked_square: None,
            promotion_result: None,
        }
    }
}

pub struct BaseBoardUI {
    piece_sprites: Option<TextureHandle>,
    sprite_state: SpriteState,

    board_rect: Rect,
    rel_mouse_pos: Pos2,
    mouse_square: Option<Square>,
    dropped_last_frame: bool,

    holding: Option<Square>,

    arrow_start: Option<Square>,
    arrows: Vec<Arrow>,

    promotion_ui: Option<PromotionUi>,
}

impl BaseBoardUI {
    pub fn new(id: Id) -> Self {
        Self {
            piece_sprites: None,
            sprite_state: SpriteState::new(id),
            board_rect: Rect::from_min_max(Pos2::ZERO, Pos2::ZERO),
            rel_mouse_pos: Pos2::ZERO,
            mouse_square: None,
            dropped_last_frame: false,
            holding: None,
            arrow_start: None,
            arrows: Vec::new(),
            promotion_ui: None,
        }
    }

    pub fn props<'a>() -> BaseBoardUIProps<'a> {
        BaseBoardUIProps::new()
    }

    pub fn ui(&mut self, ui: &mut Ui, props: BaseBoardUIProps<'_>) -> BaseBoardUIResponse {
        let board_size = Vec2::splat(SCALE * 8.0);
        let (egui_response, painter) = ui.allocate_painter(board_size, Sense::click_and_drag());

        self.board_rect = Rect::from_center_size(egui_response.rect.center(), board_size);
        if let Some(mouse_pos) = egui_response.interact_pointer_pos() {
            self.rel_mouse_pos = (mouse_pos - self.board_rect.left_top()).to_pos2();
        }
        self.mouse_square = {
            let Pos2 { x, y } = Self::render_to_board_coords(props.perspective, self.rel_mouse_pos);
            if (0.0..8.0).contains(&x) && (0.0..8.0).contains(&y) {
                Some(Square::new_unchecked(
                    y.clamp(0.0, 7.0) as _,
                    x.clamp(0.0, 7.0) as _,
                ))
            } else {
                None
            }
        };

        self.promotion_ui = props.handle_promo_on.map(|(promotion_square, for_player)| {
            PromotionUi::new(
                promotion_square,
                for_player,
                self.board_rect,
                props.perspective,
            )
        });

        self.sprite_state
            .merge_pieces(ui, props.pieces, self.dropped_last_frame);
        self.sprite_state.update(ui);
        self.dropped_last_frame = false;

        self.paint_board(&painter, &props);

        self.paint_bitboard(&painter, &props);

        self.paint_in_check(&painter, &props);

        self.paint_moves(&painter, &props);

        self.paint_pieces(ui, &props);

        self.paint_arrows(&painter, &props);

        self.paint_promotion_ui(ui, &painter);

        let mut response = BaseBoardUIResponse::new(egui_response);
        response.holding = self.holding;

        self.handle_input(&props, &mut response);

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

    fn handle_input(&mut self, props: &BaseBoardUIProps<'_>, response: &mut BaseBoardUIResponse) {
        if let Some(promotion_ui) = self.promotion_ui.as_ref() {
            response.promotion_result = promotion_ui.handle_input(&response.egui_response);
        } else {
            self.handle_drag_piece(props, response);
            self.handle_draw_arrows(props, response);
            self.handle_clicks(response);
        }
    }

    fn handle_drag_piece(
        &mut self,
        props: &BaseBoardUIProps<'_>,
        response: &mut BaseBoardUIResponse,
    ) {
        if response
            .egui_response
            .drag_started_by(PointerButton::Primary)
        {
            if let Some(start) = self.mouse_square {
                if props.drag_mask.get(start) && props.piece_at(start).is_some() {
                    self.holding = self.mouse_square;
                }
            }
        }

        if response
            .egui_response
            .drag_stopped_by(PointerButton::Primary)
        {
            if let (Some(start), Some(end)) = (self.holding, self.mouse_square) {
                if props.piece_at(start).is_some() {
                    response.dropped = Some((start, end));
                    self.dropped_last_frame = true;
                }
            }
            self.holding = None;
        }
    }

    fn handle_draw_arrows(
        &mut self,
        props: &BaseBoardUIProps<'_>,
        response: &mut BaseBoardUIResponse,
    ) {
        if props.allow_arrows
            && response
                .egui_response
                .drag_started_by(PointerButton::Secondary)
        {
            self.arrow_start = self.mouse_square;
        }

        if response
            .egui_response
            .drag_stopped_by(PointerButton::Secondary)
        {
            if let Some(start) = self.arrow_start.take() {
                let end = self.mouse_square.unwrap_or(start);
                let arrow = Arrow { start, end };

                self.toggle_arrow(arrow);
            }
        }

        if props.allow_arrows && response.egui_response.clicked_by(PointerButton::Secondary) {
            if let Some(mouse_square) = self.mouse_square {
                self.toggle_arrow(Arrow {
                    start: mouse_square,
                    end: mouse_square,
                });
            }
        }

        if response.egui_response.clicked_by(PointerButton::Primary)
            || response
                .egui_response
                .drag_stopped_by(PointerButton::Primary)
        {
            self.arrows.clear();
        }
    }

    fn toggle_arrow(&mut self, arrow: Arrow) {
        if let Some(idx) = self.arrows.iter().position(|a| a == &arrow) {
            self.arrows.swap_remove(idx);
        } else {
            self.arrows.push(arrow)
        }
    }

    fn handle_clicks(&mut self, response: &mut BaseBoardUIResponse) {
        if response.egui_response.clicked_by(PointerButton::Primary) {
            response.clicked_square = self.mouse_square;
        }
    }

    fn board_colors(&self, props: &BaseBoardUIProps<'_>) -> (Color32, Color32) {
        if props.fade_out_board {
            (BOARD_WHITE_FADED, BOARD_BLACK_FADED)
        } else {
            (BOARD_WHITE, BOARD_BLACK)
        }
    }

    fn square_is_last_move(&self, square: Square, props: &BaseBoardUIProps<'_>) -> bool {
        match props.show_last_move {
            Some((from, to)) => from == square || to == square,
            None => false,
        }
    }

    fn paint_board(&mut self, painter: &Painter, props: &BaseBoardUIProps<'_>) {
        let (white_color, black_color) = self.board_colors(props);

        painter.rect_filled(self.board_rect, 0.0, white_color);

        for square in Square::all() {
            let (rank, file) = (square.rank(), square.file());
            let rect = Self::dst_rect(square, self.board_rect, props.perspective);

            let square_is_black = (rank + file) % 2 == 0;
            if square_is_black {
                painter.rect_filled(rect, 0.0, black_color);
            }

            // Draw coordinate indicators
            let text_color = if square_is_black {
                white_color
            } else {
                black_color
            };

            let (is_visually_last_row, is_visually_last_column) = match props.perspective {
                Color::White => (rank == 0, file == 7),
                Color::Black => (rank == 7, file == 0),
            };

            if is_visually_last_row {
                painter.text(
                    rect.left_bottom() + Vec2::new(2.0, -2.0),
                    Align2::LEFT_BOTTOM,
                    (b'a' + file) as char,
                    Default::default(),
                    text_color,
                );
            }

            if is_visually_last_column {
                painter.text(
                    rect.right_top() + Vec2::new(-2.0, 2.0),
                    Align2::RIGHT_TOP,
                    (b'1' + rank) as char,
                    Default::default(),
                    text_color,
                );
            }

            if self.square_is_last_move(square, props) {
                painter.rect_filled(rect, 0.0, BOARD_LAST_MOVE);
            }
        }
    }

    fn paint_pieces(&mut self, ui: &mut Ui, props: &BaseBoardUIProps<'_>) {
        let sprite_handle = self.get_piece_sprite(ui.ctx());

        for (piece, piece_state) in self.sprite_state.get_pieces() {
            let src_rect = Self::get_piece_uv(piece);
            let (square, dst_rect) = match piece_state {
                AnimatedPieceState::Static(square) => (
                    square,
                    Self::dst_rect(square, self.board_rect, props.perspective),
                ),
                AnimatedPieceState::Moving { from, to, fraction } => {
                    let from_rect = Self::dst_rect(from, self.board_rect, props.perspective);
                    let to_rect = Self::dst_rect(to, self.board_rect, props.perspective);
                    let offset = to_rect.min - from_rect.min;

                    (to, from_rect.translate(offset * fraction))
                }
                AnimatedPieceState::Emphemeral { on, .. } => {
                    (on, Self::dst_rect(on, self.board_rect, props.perspective))
                }
            };

            let mut image = egui::Image::new(&sprite_handle).uv(src_rect);

            if let AnimatedPieceState::Emphemeral { fraction, .. } = piece_state {
                image = image.tint(Rgba::from_rgba_unmultiplied(1.0, 1.0, 1.0, 1.0 - fraction));
            } else if self.holding == Some(square) {
                image = image.tint(Rgba::from_rgba_unmultiplied(1.0, 1.0, 1.0, 0.2));
            }

            image.paint_at(ui, dst_rect)
        }

        if let Some(drag_from_square) = self.holding {
            if let Some(dragged_piece) = props
                .pieces
                .get(drag_from_square.index())
                .copied()
                .flatten()
            {
                let src_rect = Self::get_piece_uv(dragged_piece);
                let dst_rect = Rect::from_center_size(
                    self.board_rect.left_top() + self.rel_mouse_pos.to_vec2(),
                    Vec2::splat(SCALE),
                );

                egui::Image::new(&sprite_handle)
                    .uv(src_rect)
                    .paint_at(ui, dst_rect);
            } else {
                // Dragged piece was removed from underneath us...
            }
        }
    }

    fn paint_moves(&mut self, painter: &Painter, props: &BaseBoardUIProps<'_>) {
        let mut start_squares = [false; 64];
        let mut end_squares = [false; 64];

        for (from, to) in props.possible_moves {
            start_squares[from.index()] = true;
            end_squares[to.index()] = true;
        }

        for square in Square::all() {
            let is_start_square = start_squares[square.index()];
            let is_end_square = end_squares[square.index()];

            if !is_start_square && !is_end_square {
                continue;
            }

            let dst_rect = Self::dst_rect(square, self.board_rect, props.perspective);
            if is_start_square || self.mouse_square == Some(square) {
                painter.add(RectShape::filled(dst_rect, 0.0, MOVE_COLOR));
            } else if props.pieces[square.index()].is_some() {
                let triangles = [
                    vec![
                        dst_rect.left_top(),
                        dst_rect.left_top() + vec2(SCALE * 0.25, 0.0),
                        dst_rect.left_top() + vec2(0.0, SCALE * 0.25),
                    ],
                    vec![
                        dst_rect.right_top(),
                        dst_rect.right_top() + vec2(-SCALE * 0.25, 0.0),
                        dst_rect.right_top() + vec2(0.0, SCALE * 0.25),
                    ],
                    vec![
                        dst_rect.left_bottom(),
                        dst_rect.left_bottom() + vec2(SCALE * 0.25, 0.0),
                        dst_rect.left_bottom() + vec2(0.0, -SCALE * 0.25),
                    ],
                    vec![
                        dst_rect.right_bottom(),
                        dst_rect.right_bottom() + vec2(-SCALE * 0.25, 0.0),
                        dst_rect.right_bottom() + vec2(1.0, -SCALE * 0.25),
                    ],
                ];

                for points in triangles {
                    painter.add(PathShape {
                        points,
                        closed: true,
                        fill: MOVE_COLOR,
                        stroke: PathStroke::NONE,
                    });
                }
            } else {
                painter.circle_filled(dst_rect.center(), SCALE * 0.125, MOVE_COLOR);
            }
        }
    }

    fn paint_in_check(&mut self, painter: &Painter, props: &BaseBoardUIProps<'_>) {
        if let Some(square) = props.checked_king_position {
            painter.add(
                Shadow {
                    blur: SCALE * 0.25,
                    spread: -SCALE * 0.125,
                    color: Color32::RED,
                    ..Default::default()
                }
                .as_shape(
                    Self::dst_rect(square, self.board_rect, props.perspective),
                    SCALE * 0.5,
                ),
            );
        }
    }

    fn paint_bitboard(&mut self, painter: &Painter, props: &BaseBoardUIProps<'_>) {
        for square in Square::all() {
            if props.display_bitboard.get(square) {
                let rect = Self::dst_rect(square, self.board_rect, props.perspective);
                painter.rect_filled(rect, 0.0, BOARD_BITBOARD_HIGHLIGHT);
            }
        }
    }

    fn paint_arrows(&mut self, painter: &Painter, props: &BaseBoardUIProps<'_>) {
        for arrow in self.arrows.iter() {
            arrow.draw(painter, self.board_rect, props.perspective, false);
        }

        if let Some(start) = self.arrow_start {
            let end = match self.mouse_square {
                Some(end) => end,
                None => start,
            };
            Arrow { start, end }.draw(painter, self.board_rect, props.perspective, true);
        }
    }

    fn paint_promotion_ui(&mut self, ui: &mut Ui, painter: &Painter) {
        let texture_handle = self.get_piece_sprite(ui.ctx());
        if let Some(promotion_ui) = self.promotion_ui.as_ref() {
            promotion_ui.draw(ui, painter, &texture_handle);
        }
    }
}
