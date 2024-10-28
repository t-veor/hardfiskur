use egui::{Id, Ui};
use hardfiskur_core::board::{Bitboard, Board, Color, Move, Piece, Square};

use crate::{
    base_board::{BaseBoardUI, BaseBoardUIProps, BaseBoardUIResponse, PromotionResult},
    constants::MIN_BOARD_SIZE,
};

#[derive(Debug)]
pub struct ChessBoardUIProps<'a> {
    board: &'a Board,
    can_move: bool,
    perspective: Color,
    fade_out_board: bool,
    show_last_move: Option<(Square, Square)>,

    // min, max
    board_size: (Option<f32>, Option<f32>),
}

impl<'a> ChessBoardUIProps<'a> {
    pub fn new(board: &'a Board) -> Self {
        Self {
            board,
            can_move: false,
            perspective: Color::White,
            fade_out_board: false,
            show_last_move: None,
            board_size: (None, Some(640.0)),
        }
    }

    pub fn can_move(mut self, can_move: bool) -> Self {
        self.can_move = can_move;
        self
    }

    pub fn perspective(mut self, perspective: Color) -> Self {
        self.perspective = perspective;
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

    pub fn min_size(mut self, min_size: f32) -> Self {
        self.board_size.0 = Some(min_size);
        self
    }

    pub fn max_size(mut self, max_size: f32) -> Self {
        self.board_size.1 = Some(max_size);
        self
    }

    pub fn exact_size(mut self, size: f32) -> Self {
        self.board_size = (Some(size), Some(size));
        self
    }
}

#[derive(Debug)]
pub struct ChessBoardResponse {
    pub egui_response: egui::Response,
    pub input_move: Option<Move>,
}

pub struct ChessBoardUI {
    base_board: BaseBoardUI,

    selected: Option<Square>,

    promotion_progress: Option<((Square, Square), Color)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum HandleMoveResult {
    None,
    PromotionBuffered,
    FoundMove(Move),
}

impl ChessBoardUI {
    pub fn new(id: Id) -> Self {
        Self {
            base_board: BaseBoardUI::new(id),
            selected: None,
            promotion_progress: None,
        }
    }

    pub fn props(board: &Board) -> ChessBoardUIProps {
        ChessBoardUIProps::new(board)
    }

    pub fn ui(&mut self, ui: &mut Ui, props: ChessBoardUIProps<'_>) -> ChessBoardResponse {
        let board = props.board;
        let can_move = props.can_move;
        let pieces = self.get_pieces(props.board);
        let (moves, move_gen_res) = props.board.legal_moves_and_meta();
        let in_check = move_gen_res.checker_count > 0;

        let mut possible_moves = Vec::new();

        if let Some(selected) = self.selected {
            for m in moves.iter() {
                if m.from_square() == selected {
                    possible_moves.push((m.from_square(), m.to_square()));

                    // Also display that the king can "capture" the rook for a
                    // castling move.
                    if m.is_castle() {
                        possible_moves.push((m.from_square(), m.castling_rook_squares().0));
                    }
                }
            }
        }

        let base_board_data =
            self.gather_baseboard_props(ui, props, &pieces, &possible_moves, in_check);

        let base_board_response = self.base_board.ui(ui, base_board_data);

        self.handle_baseboard_response(base_board_response, board, can_move, &moves)
    }

    fn get_pieces(&self, board: &Board) -> [Option<Piece>; 64] {
        let mut pieces = [None; 64];

        for (piece, square) in board.pieces() {
            pieces[square.index()] = Some(piece);
        }

        pieces
    }

    fn gather_baseboard_props<'a>(
        &mut self,
        ui: &Ui,
        props: ChessBoardUIProps<'_>,
        pieces: &'a [Option<Piece>],
        possible_moves: &'a [(Square, Square)],
        in_check: bool,
    ) -> BaseBoardUIProps<'a> {
        let ChessBoardUIProps {
            board,
            can_move,
            perspective,
            fade_out_board,
            show_last_move: last_move,

            board_size,
        } = props;

        let board_size = {
            let available_size = ui.available_size();
            let mut size = available_size.x.min(available_size.y);

            if let Some(min_size) = board_size.0 {
                size = size.max(min_size);
            }

            if let Some(max_size) = board_size.1 {
                size = size.min(max_size)
            }

            size = size.max(MIN_BOARD_SIZE);
            size
        };

        let mut base_props = BaseBoardUI::props()
            .pieces(pieces)
            .possible_moves(possible_moves)
            .perspective(perspective)
            .drag_mask(if can_move {
                board.get_bitboard_for_color(board.to_move())
            } else {
                Bitboard::EMPTY
            })
            .fade_out_board(fade_out_board)
            .with_size(board_size);

        if let Some(((_start, end), color)) = self.promotion_progress {
            base_props = base_props.handle_promo_on(end, color);
        }

        if in_check {
            base_props = base_props.checked_king_position(Some(board.get_king(board.to_move())));
        }

        if let Some((from, to)) = last_move {
            base_props = base_props.show_last_move(from, to);
        }

        base_props
    }

    fn handle_baseboard_response(
        &mut self,
        base_response: BaseBoardUIResponse,
        board: &Board,
        can_move: bool,
        moves: &[Move],
    ) -> ChessBoardResponse {
        let mut response = ChessBoardResponse {
            egui_response: base_response.egui_response,
            input_move: None,
        };

        if !can_move {
            self.selected = None;
            return response;
        }

        // Handle promotions
        if let Some(m) = self.handle_promotion(base_response.promotion_result, moves) {
            response.input_move = Some(m);
            return response;
        }

        // Handle clicks
        match (self.selected, base_response.clicked_square) {
            // Same square clicked again.
            (Some(from), Some(to)) if from == to => self.selected = None,

            (Some(from), Some(to)) => match self.handle_possible_move(Some((from, to)), moves) {
                HandleMoveResult::PromotionBuffered => {
                    return response;
                }
                HandleMoveResult::FoundMove(m) => {
                    response.input_move = Some(m);
                    return response;
                }
                HandleMoveResult::None => {
                    self.selected = match board.get_piece(to) {
                        Some(piece) if piece.color() == board.to_move() => Some(to),
                        _ => None,
                    }
                }
            },
            (None, Some(clicked)) => self.selected = Some(clicked),

            _ => (),
        }

        // Handle drags
        match self.handle_possible_move(base_response.dropped, moves) {
            HandleMoveResult::FoundMove(m) => {
                response.input_move = Some(m);
                return response;
            }
            HandleMoveResult::PromotionBuffered => return response,
            HandleMoveResult::None => (),
        };
        self.selected = base_response.holding.or(self.selected);

        response
    }

    fn handle_possible_move(
        &mut self,
        dropped: Option<(Square, Square)>,
        moves: &[Move],
    ) -> HandleMoveResult {
        let (start, end) = match dropped {
            Some(x) => x,
            None => return HandleMoveResult::None,
        };
        let found_move = match moves.iter().find(|m| {
            m.from_square() == start
                && (m.to_square() == end
                    // Allow "capturing" the rook for a castling move.
                    || m.is_castle() && m.castling_rook_squares().0 == end)
        }) {
            Some(m) => *m,
            None => return HandleMoveResult::None,
        };

        // Clear the selection since we're making a valid move
        self.selected = None;

        // Check if the move is a promotion. If it is, buffer the promotion and
        // don't finish until the user is finished with selecting the promotion
        if found_move.promotion().is_some() {
            self.promotion_progress = Some(((start, end), found_move.piece().color()));
            HandleMoveResult::PromotionBuffered
        } else {
            HandleMoveResult::FoundMove(found_move)
        }
    }

    fn handle_promotion(
        &mut self,
        promotion_result: Option<PromotionResult>,
        possible_moves: &[Move],
    ) -> Option<Move> {
        let promotion_result = promotion_result?;

        // This take means that if a promotion_result is Cancel, the
        // promotion_progress will be cleared
        let ((start, end), color) = self.promotion_progress.take()?;

        // (Bails if promotion_result is Cancel)
        let piece_type = promotion_result.into_piece_type()?;

        let promotion = Piece::new(color, piece_type);

        possible_moves
            .iter()
            .find(|m| {
                m.from_square() == start && m.to_square() == end && m.promotion() == Some(promotion)
            })
            .copied()
    }
}
