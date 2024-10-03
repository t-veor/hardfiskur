use egui::{Id, Ui};
use hardfiskur_core::board::{Bitboard, Board, Color, Move, Piece, Square};

use crate::base_board::{BaseBoardUI, BaseBoardUIProps, BaseBoardUIResponse, PromotionResult};

#[derive(Debug)]
pub struct ChessBoardData<'a> {
    pub board: &'a Board,
    pub can_move: bool,
    pub perspective: Color,
    pub fade_out_board: bool,
    pub last_move: Option<(Square, Square)>,
}

#[derive(Debug)]
pub struct ChessBoardResponse {
    pub egui_response: egui::Response,
    pub input_move: Option<Move>,
}

pub struct ChessBoard {
    base_board: BaseBoardUI,

    holding: Option<Square>,

    promotion_progress: Option<((Square, Square), Color)>,
}

impl ChessBoard {
    pub fn new(id: Id) -> Self {
        Self {
            base_board: BaseBoardUI::new(id),
            holding: None,
            promotion_progress: None,
        }
    }

    pub fn ui(&mut self, ui: &mut Ui, data: ChessBoardData<'_>) -> ChessBoardResponse {
        let pieces = self.get_pieces(data.board);
        let (moves, move_gen_res) = data.board.legal_moves_and_meta();
        let in_check = move_gen_res.checker_count > 0;

        let mut possible_moves = Vec::new();

        if let Some(holding) = self.holding {
            for m in moves.iter() {
                if m.from_square() == holding {
                    possible_moves.push((m.from_square(), m.to_square()));

                    // Also display that the king can "capture" the rook for a
                    // castling move.
                    if m.is_castle() {
                        possible_moves.push((m.from_square(), m.castling_rook_squares().0));
                    }
                }
            }
        }

        let base_board_data = self.gather_baseboard_props(data, &pieces, &possible_moves, in_check);

        let base_board_response = self.base_board.ui(ui, base_board_data);

        self.handle_baseboard_response(base_board_response, &moves)
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
        data: ChessBoardData<'_>,
        pieces: &'a [Option<Piece>],
        possible_moves: &'a [(Square, Square)],
        in_check: bool,
    ) -> BaseBoardUIProps<'a> {
        let ChessBoardData {
            board,
            can_move,
            perspective,
            fade_out_board,
            last_move,
        } = data;

        let mut base_props = BaseBoardUI::props()
            .pieces(pieces)
            .possible_moves(&possible_moves)
            .perspective(perspective)
            .drag_mask(if can_move {
                board.get_bitboard_for_color(board.to_move())
            } else {
                Bitboard::EMPTY
            })
            .fade_out_board(fade_out_board);

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
        response: BaseBoardUIResponse,
        moves: &[Move],
    ) -> ChessBoardResponse {
        self.holding = response.holding;

        let mut input_move = self.handle_possible_move(response.dropped, moves);
        input_move = input_move.or(self.handle_promotion(response.promotion_result, moves));

        ChessBoardResponse {
            egui_response: response.egui_response,
            input_move,
        }
    }

    fn handle_possible_move(
        &mut self,
        dropped: Option<(Square, Square)>,
        moves: &[Move],
    ) -> Option<Move> {
        let (start, end) = dropped?;
        let found_move = *moves.iter().find(|m| {
            m.from_square() == start
                && (m.to_square() == end
                    // Allow "capturing" the rook for a castling move.
                    || m.is_castle() && m.castling_rook_squares().0 == end)
        })?;

        // Check if the move is a promotion. If it is, buffer the promotion and
        // don't finish until the user is finished with selecting the promotion
        if found_move.promotion().is_some() {
            self.promotion_progress = Some(((start, end), found_move.piece().color()));
            None
        } else {
            Some(found_move)
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
