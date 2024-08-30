use eframe::egui::{self, Layout, Vec2};
use hardfiskur_core::board::{Board, Color};
use hardfiskur_ui::chess_board::{ChessBoard, ChessBoardData};

struct HardfiskurUI {
    chess_ui: ChessBoard,
    board: Board,
}

impl HardfiskurUI {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            chess_ui: Default::default(),
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
