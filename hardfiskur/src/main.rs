use std::{io::Cursor, usize};

use eframe::egui::{self, Id, Key, Layout, Vec2};
use hardfiskur_core::board::{Board, Color, Move, Piece, PieceType};
use hardfiskur_ui::chess_board::{ChessBoard, ChessBoardData};
use rand::prelude::*;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Source};

struct HardfiskurUI {
    chess_ui: ChessBoard,
    board: Board,

    output_stream: OutputStream,
    output_stream_handle: OutputStreamHandle,
}

impl HardfiskurUI {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let (stream, handle) = OutputStream::try_default().unwrap();

        Self {
            chess_ui: ChessBoard::new(Id::new("hardfiskur_ui_board")),
            board: Board::starting_position(),

            output_stream: stream,
            output_stream_handle: handle,
        }
    }

    fn make_move(&mut self, the_move: Move) {
        let move_sound = Decoder::new(Cursor::new(include_bytes!("Move.ogg").as_slice()))
            .unwrap()
            .convert_samples();
        let capture_sound = Decoder::new(Cursor::new(include_bytes!("Capture.ogg").as_slice()))
            .unwrap()
            .convert_samples();

        // TODO: this is horrible
        if the_move.is_capture() {
            self.output_stream_handle.play_raw(capture_sound).unwrap();
        } else {
            self.output_stream_handle.play_raw(move_sound).unwrap();
        }

        self.board.push_move_repr(the_move);
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

        egui::Window::new("Actions").show(ctx, |ui| {
            if ui.button("Random move").clicked()
                || ctx.input(|input| input.key_pressed(Key::Space))
            {
                if let Some(the_move) = random_move(&self.board) {
                    self.make_move(the_move);
                }
            }

            if ui.button("Reset").clicked() {
                self.board = Board::starting_position();
            }
        });
    }
}

fn random_move(board: &Board) -> Option<Move> {
    let (legal_moves, _) = board.legal_moves();
    legal_moves.choose(&mut thread_rng()).copied()
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
