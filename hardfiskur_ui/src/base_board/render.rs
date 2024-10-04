use egui::{Align2, Color32, Painter, Pos2, Rect, Shadow, Vec2};
use hardfiskur_core::board::{Bitboard, Color, Piece, PieceType, Square};

use crate::board_style::BoardStyle;

use super::sprite_state::SpriteState;

pub struct BoardRenderContext<'a> {
    pub painter: &'a Painter,
    pub style: &'a BoardStyle,
    pub board_rect: Rect,
    pub sprite_state: &'a SpriteState,
    pub perspective: Color,
    pub last_move: Option<(Square, Square)>,
}

impl<'a> BoardRenderContext<'a> {
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

    fn square_is_last_move(&self, square: Square) -> bool {
        match self.last_move {
            Some((from, to)) => from == square || to == square,
            None => false,
        }
    }

    pub fn paint_board(&self, faded: bool) {
        let (white_color, black_color) = self.style.board_colors(faded);

        self.painter.rect_filled(self.board_rect, 0.0, white_color);

        for square in Square::all() {
            let (rank, file) = (square.rank(), square.file());
            let rect = self
                .style
                .board_square(square, self.board_rect, self.perspective);

            let square_is_black = (rank + file) % 2 == 0;
            if square_is_black {
                self.painter.rect_filled(rect, 0.0, black_color);
            }

            // Draw coordinate indicators
            let text_color = if square_is_black {
                white_color
            } else {
                black_color
            };

            let (is_visually_last_row, is_visually_last_column) = match self.perspective {
                Color::White => (rank == 0, file == 7),
                Color::Black => (rank == 7, file == 0),
            };

            if is_visually_last_row {
                self.painter.text(
                    rect.left_bottom() + Vec2::new(2.0, -2.0),
                    Align2::LEFT_BOTTOM,
                    (b'a' + file) as char,
                    Default::default(),
                    text_color,
                );
            }

            if is_visually_last_column {
                self.painter.text(
                    rect.right_top() + Vec2::new(-2.0, 2.0),
                    Align2::RIGHT_TOP,
                    (b'1' + rank) as char,
                    Default::default(),
                    text_color,
                );
            }

            if self.square_is_last_move(square) {
                self.painter
                    .rect_filled(rect, 0.0, self.style.last_move_color());
            }
        }
    }

    pub fn paint_bitboard(&self, bitboard: Bitboard) {
        for square in bitboard.squares() {
            let rect = self
                .style
                .board_square(square, self.board_rect, self.perspective);
            self.painter
                .rect_filled(rect, 0.0, self.style.bitboard_highlight_color());
        }
    }

    pub fn paint_check_indicator(&self, checked_king_position: Option<Square>) {
        if let Some(square) = checked_king_position {
            let square_size = self.style.square_size;

            self.painter.add(
                Shadow {
                    blur: square_size * 0.25,
                    spread: -square_size * 0.125,
                    color: Color32::RED,
                    ..Default::default()
                }
                .as_shape(
                    self.style
                        .board_square(square, self.board_rect, self.perspective),
                    square_size * 0.5,
                ),
            );
        }
    }
}
