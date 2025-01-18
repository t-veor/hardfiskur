mod board_manager;
mod fen_input;
mod search_thread;
mod sfx_stream;

use std::time::Duration;

use eframe::egui::{self, Layout, Vec2};
use hardfiskur_core::board::{Board, Move};

use board_manager::BoardManager;
use fen_input::FenInput;
use search_thread::SearchThread;
use sfx_stream::SFXStream;

struct HardfiskurApp {
    board_manager: BoardManager,

    fen_input: FenInput,

    move_time: Duration,
    search_thread: SearchThread,
    sfx_stream: SFXStream,

    automove_after_user: bool,
    automove_after_engine: bool,
}

impl HardfiskurApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            board_manager: BoardManager::new(),

            fen_input: FenInput::new(),

            search_thread: SearchThread::new(),
            move_time: Duration::from_secs(1),
            sfx_stream: SFXStream::new(),

            automove_after_user: false,
            automove_after_engine: false,
        }
    }

    fn start_search(&mut self, ctx: &egui::Context) {
        if !self.board_manager.playing() {
            return;
        }

        if !self.search_thread.searching() {
            let ctx = ctx.clone();
            self.search_thread.send_search_request(
                self.board_manager.current_board(),
                self.move_time,
                move || {
                    ctx.request_repaint();
                },
            );
        }
    }

    fn make_move(&mut self, ctx: &egui::Context, the_move: Move, from_user: bool) {
        if self.board_manager.push_move(the_move) {
            if the_move.is_capture() {
                self.sfx_stream.play_capture();
            } else {
                self.sfx_stream.play_move();
            }
        }

        self.search_thread.cancel_search();

        if from_user && self.automove_after_user || !from_user && self.automove_after_engine {
            self.start_search(ctx);
        }
    }
}

impl eframe::App for HardfiskurApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(m) = self.search_thread.try_receive_move() {
            self.make_move(ctx, m, false);
        }

        egui::SidePanel::right("right_panel")
            .resizable(false)
            .min_width(200.0)
            .show(ctx, |ui| {
                if ui.button("Make move").clicked() && self.board_manager.playing() {
                    self.start_search(ctx);
                }

                if ui.button("Reset").clicked() {
                    self.board_manager.reset();
                    self.search_thread.reset();
                }

                if ui.button("Undo move").clicked() {
                    self.board_manager.pop_move();
                }

                let mut move_time_secs = self.move_time.as_secs_f64();
                ui.add(
                    egui::DragValue::new(&mut move_time_secs)
                        .prefix("Move time: ")
                        .speed(0.1)
                        .range(0.0..=600.0)
                        .clamp_to_range(false)
                        .suffix(" secs"),
                );
                self.move_time =
                    Duration::try_from_secs_f64(move_time_secs).unwrap_or(Duration::ZERO);

                ui.checkbox(&mut self.automove_after_user, "Move after user");
                ui.checkbox(&mut self.automove_after_engine, "Move again after engine");

                ui.separator();

                if let Some(scroll_request) = self.board_manager.ui_move_history(ui) {
                    self.board_manager.scroll_to(scroll_request);
                }
            });

        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            if let Some(new_fen) = self
                .fen_input
                .show(ui, &self.board_manager.current_board().fen())
            {
                if let Ok(board) = Board::try_parse_fen(&new_fen) {
                    self.board_manager.reset_to(board);
                }
            }

            ui.label(format!(
                "{:?}",
                self.board_manager.current_board().zobrist_hash()
            ));
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(
                Layout::centered_and_justified(egui::Direction::LeftToRight),
                |ui| {
                    let input_move = self.board_manager.ui_board(ui);

                    if let Some(m) = input_move {
                        self.make_move(ctx, m, true);
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
        Box::new(|cc| Ok(Box::new(HardfiskurApp::new(cc)))),
    )
}
