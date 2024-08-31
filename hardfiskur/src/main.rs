use std::{io::Cursor, usize};

use eframe::egui::{self, Id, Key, Layout, Vec2};
use hardfiskur_core::board::{Board, Color, Move, Piece, PieceType};
use hardfiskur_ui::chess_board::{ChessBoard, ChessBoardData};
use rand::prelude::*;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Source};

struct HardfiskurUI {
    chess_ui: ChessBoard,
    board: Board,

    move_history: Vec<String>,

    _output_stream: OutputStream,
    output_stream_handle: OutputStreamHandle,
}

impl HardfiskurUI {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let (stream, handle) = OutputStream::try_default().unwrap();

        Self {
            chess_ui: ChessBoard::new(Id::new("hardfiskur_ui_board")),
            board: Board::starting_position(),

            move_history: vec![],

            _output_stream: stream,
            output_stream_handle: handle,
        }
    }

    fn reset(&mut self) {
        self.board = Board::starting_position();
        self.move_history.clear();
    }

    fn make_move(&mut self, the_move: Move) {
        let san = self.board.get_san(the_move);

        if self.board.push_move_repr(the_move) {
            let sound_file = if the_move.is_capture() {
                include_bytes!("Capture.ogg").as_slice()
            } else {
                include_bytes!("Move.ogg").as_slice()
            };
            let sound = Decoder::new(Cursor::new(sound_file))
                .unwrap()
                .convert_samples();

            self.output_stream_handle.play_raw(sound).unwrap();

            self.move_history.push(match san {
                Some(san) => format!("{san}"),
                None => "?".to_string(),
            });
        }
    }
}

impl eframe::App for HardfiskurUI {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(
                Layout::centered_and_justified(egui::Direction::LeftToRight),
                |ui| {
                    let response = self.chess_ui.ui(
                        ui,
                        ChessBoardData {
                            board: &self.board,
                            can_move: true,
                            perspective: Color::White,
                        },
                    );

                    if let Some(m) = response.input_move {
                        self.make_move(m);
                    }
                },
            );
        });

        egui::SidePanel::right("right_panel").show(ctx, |ui| {
            if ui.button("Random move").clicked()
                || ctx.input(|input| input.key_pressed(Key::Space))
            {
                if self.board.to_move().is_white() {
                    if let Some(the_move) = min_king_distance(&self.board) {
                        self.make_move(the_move);
                    }
                } else {
                    if let Some(the_move) = minimize_opp_moves2(&self.board) {
                        self.make_move(the_move);
                    }
                }
            }

            if ui.button("Reset").clicked() {
                self.reset();
            }

            egui::Grid::new("moves").show(ui, |ui| {
                for (i, m) in self.move_history.iter().enumerate() {
                    ui.label(m);
                    if i % 2 == 1 {
                        ui.end_row();
                    }
                }
            });
        });
    }
}

fn random_move(board: &Board) -> Option<Move> {
    let legal_moves = board.legal_moves();
    legal_moves.choose(&mut rand::thread_rng()).copied()
}

fn minimize_opp_moves2(board: &Board) -> Option<Move> {
    let mut board = board.clone();
    let legal_moves = board.legal_moves();

    let mut best_moves = Vec::new();
    let mut min_opp_moves = usize::MAX;

    'outer: for m in legal_moves.iter().copied() {
        board.push_move_unchecked(m);
        let (opp_m, r) = board.legal_moves_and_checkers();
        let checkers = r.checker_count;
        let opp_moves = opp_m.len();
        board.pop_move();

        // avoid stalemate
        if checkers == 0 && opp_moves == 0 {
            continue;
        }

        // avoid recaptures
        for om in opp_m {
            if om.to_square() == m.to_square() {
                continue 'outer;
            }
        }

        if opp_moves < min_opp_moves {
            best_moves = vec![m];
            min_opp_moves = opp_moves;
        } else if opp_moves == min_opp_moves {
            best_moves.push(m)
        }
    }

    best_moves
        .choose(&mut rand::thread_rng())
        .copied()
        .or_else(|| minimize_opp_moves(&board))
}

fn minimize_opp_moves(board: &Board) -> Option<Move> {
    let mut board = board.clone();
    let legal_moves = board.legal_moves();

    let mut best_moves = Vec::new();
    let mut min_opp_moves = usize::MAX;

    for m in legal_moves.iter().copied() {
        board.push_move_unchecked(m);
        let (opp_m, r) = board.legal_moves_and_checkers();
        let checkers = r.checker_count;
        let opp_moves = opp_m.len();
        board.pop_move();

        // avoid stalemate
        if checkers == 0 && opp_moves == 0 {
            continue;
        }

        if opp_moves < min_opp_moves {
            best_moves = vec![m];
            min_opp_moves = opp_moves;
        } else if opp_moves == min_opp_moves {
            best_moves.push(m)
        }
    }

    best_moves
        .choose(&mut rand::thread_rng())
        .copied()
        .or_else(|| random_move(&board))
}

fn min_king_distance(board: &Board) -> Option<Move> {
    let mut board = board.clone();
    let legal_moves = board.legal_moves();

    let mut best_moves = Vec::new();
    let mut min_distance = u32::MAX;

    for m in legal_moves.iter().copied() {
        board.push_move_unchecked(m);

        let white_king = board
            .pieces()
            .find(|(piece, square)| *piece == Piece::WHITE_KING)
            .unwrap()
            .1;
        let black_king = board
            .pieces()
            .find(|(piece, square)| *piece == Piece::BLACK_KING)
            .unwrap()
            .1;

        let king_distance = white_king.euclidean_distance_sq(black_king);

        board.pop_move();

        if king_distance < min_distance {
            best_moves = vec![m];
            min_distance = king_distance;
        } else if king_distance == min_distance {
            best_moves.push(m)
        }
    }

    best_moves
        .choose(&mut rand::thread_rng())
        .copied()
        .or_else(|| random_move(&board))
}

fn pawns_only(board: &Board) -> Option<Move> {
    let legal_moves = board.legal_moves();

    let pawn_captures: Vec<_> = legal_moves
        .iter()
        .filter(|m| m.piece().piece_type() == PieceType::Pawn && m.is_capture())
        .copied()
        .collect();

    let pawn_moves: Vec<_> = legal_moves
        .iter()
        .filter(|m| m.piece().piece_type() == PieceType::Pawn)
        .copied()
        .collect();

    pawn_captures
        .choose(&mut rand::thread_rng())
        .copied()
        .or_else(|| {
            pawn_moves
                .choose(&mut rand::thread_rng())
                .copied()
                .or_else(|| minimize_opp_moves(&board))
        })
}

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "Harðfiskur",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size(Vec2::new(1024.0, 768.0)),
            ..Default::default()
        },
        Box::new(|cc| Ok(Box::new(HardfiskurUI::new(cc)))),
    )
}
