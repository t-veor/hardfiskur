use egui::Ui;
use hardfiskur_core::board::{Bitboard, Board, Color, Move, Piece, PieceType, Square};

use crate::base_board::{BaseBoard, BaseBoardData, BaseBoardResponse, PromotionResult};

#[derive(Debug)]
pub struct ChessBoardData<'a> {
    pub board: &'a Board,
    pub can_move: bool,
    pub perspective: Color,
}

#[derive(Debug)]
pub struct ChessBoardResponse {
    pub egui_response: egui::Response,
    pub input_move: Option<Move>,
}

#[derive(Default)]
pub struct ChessBoard {
    base_board: BaseBoard,

    holding: Option<Square>,

    promotion_progress: Option<((Square, Square), Color)>,
}

impl ChessBoard {
    pub fn ui(&mut self, ui: &mut Ui, data: ChessBoardData<'_>) -> ChessBoardResponse {
        let pieces = self.get_pieces(data.board);
        let (moves, move_gen_res) = data.board.legal_moves();
        let in_check = move_gen_res.checker_count > 0;

        let possible_moves = if let Some(holding) = self.holding {
            moves
                .iter()
                .filter(|m| m.from_square() == holding)
                .copied()
                .collect()
        } else {
            Vec::new()
        };

        let base_board_data = self.gather_baseboard_data(data, &pieces, &possible_moves, in_check);

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

    fn gather_baseboard_data<'a>(
        &mut self,
        data: ChessBoardData<'_>,
        pieces: &'a [Option<Piece>],
        possible_moves: &'a [Move],
        in_check: bool,
    ) -> BaseBoardData<'a> {
        let ChessBoardData {
            board,
            can_move,
            perspective,
        } = data;

        let drag_mask = if can_move {
            board.color_bitboard(board.to_move())
        } else {
            Bitboard::EMPTY
        };

        let checked_king_position = if in_check {
            pieces
                .iter()
                .position(|p| *p == Some(Piece::new(board.to_move(), PieceType::King)))
                .and_then(Square::from_index)
        } else {
            None
        };

        BaseBoardData {
            pieces,
            possible_moves,
            perspective,
            drag_mask,

            promotion: self
                .promotion_progress
                .map(|((_start, end), color)| (end, color)),

            checked_king_position,

            ..Default::default()
        }
    }

    fn handle_baseboard_response(
        &mut self,
        response: BaseBoardResponse,
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
        let found_move = *moves
            .iter()
            .find(|m| m.from_square() == start && m.to_square() == end)?;

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
