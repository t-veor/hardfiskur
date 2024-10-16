mod fen_input;
mod game_manager;
mod search_thread;
mod sfx_stream;

use eframe::egui::{self, Layout, Vec2};
use fen_input::FenInput;
use game_manager::GameManager;
use hardfiskur_core::board::{Board, Move};

use search_thread::SearchThread;
use sfx_stream::SFXStream;

struct HardfiskurApp {
    game_manager: GameManager,

    fen_input: FenInput,

    search_thread: SearchThread,
    sfx_stream: SFXStream,

    user_just_moved: bool,
}

impl HardfiskurApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            game_manager: GameManager::new(),

            fen_input: FenInput::new(),

            search_thread: SearchThread::new(),
            sfx_stream: SFXStream::new(),

            user_just_moved: false,
        }
    }

    fn start_search(&mut self, ctx: &egui::Context) {
        if !self.search_thread.searching() {
            let ctx = ctx.clone();
            self.search_thread
                .send_search_request(self.game_manager.current_board(), move || {
                    ctx.request_repaint();
                });
        }
    }

    fn make_move(&mut self, the_move: Move) {
        if self.game_manager.push_move(the_move) {
            if the_move.is_capture() {
                self.sfx_stream.play_capture();
            } else {
                self.sfx_stream.play_move();
            }
        }

        self.search_thread.cancel_search();
    }
}

impl eframe::App for HardfiskurApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(m) = self.search_thread.try_receive_move() {
            self.make_move(m);
        }

        egui::SidePanel::right("right_panel")
            .resizable(false)
            .min_width(200.0)
            .show(ctx, |ui| {
                if ui.button("Make move").clicked() && self.game_manager.playing() {
                    self.start_search(ctx);
                }

                if ui.button("Reset").clicked() {
                    self.game_manager.reset();
                    self.search_thread.reset();
                }

                if ui.button("Undo move").clicked() {
                    self.game_manager.pop_move();
                }

                ui.separator();

                if let Some(scroll_request) = self.game_manager.ui_move_history(ui) {
                    self.game_manager.scroll_to(scroll_request);
                }
            });

        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            if let Some(new_fen) = self
                .fen_input
                .show(ui, &self.game_manager.current_board().fen())
            {
                if let Ok(board) = Board::try_parse_fen(&new_fen) {
                    self.game_manager.reset_to(board);
                }
            }

            ui.label(format!(
                "{:?}",
                self.game_manager.current_board().zobrist_hash()
            ));
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(
                Layout::centered_and_justified(egui::Direction::LeftToRight),
                |ui| {
                    let input_move = self.game_manager.ui_board(ui);

                    self.user_just_moved = false;

                    if let Some(m) = input_move {
                        self.make_move(m);
                        self.user_just_moved = true;
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
