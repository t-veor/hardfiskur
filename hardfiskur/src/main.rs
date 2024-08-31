use eframe::egui::{self, Id, Layout, Vec2};
use hardfiskur_core::board::{Board, Color};
use hardfiskur_ui::chess_board::{ChessBoard, ChessBoardData};
use rand::prelude::*;

struct HardfiskurUI {
    chess_ui: ChessBoard,
    board: Board,
}

impl HardfiskurUI {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            chess_ui: ChessBoard::new(Id::new("hardfiskur_ui_board")),
            board: Board::starting_position(),
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
                        self.board.push_move_repr(m);
                    }
                },
            );
        });

        egui::Window::new("Actions").show(ctx, |ui| {
            if ui.button("Random move").clicked() {
                let (legal_moves, _) = self.board.legal_moves();
                if let Some(the_move) = legal_moves.choose(&mut rand::thread_rng()) {
                    self.board.push_move_repr(*the_move);
                }
            }

            if ui.button("Reset").clicked() {
                self.board = Board::starting_position();
            }
        });
    }
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
