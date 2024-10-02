mod game_manager;
mod search_thread;
mod sfx_stream;

use eframe::egui::{self, Key, Layout, Vec2};
use game_manager::GameManager;
use hardfiskur_core::board::{BoardState, Color, DrawReason, Move};

use search_thread::SearchThread;
use sfx_stream::SFXStream;

struct HardfiskurApp {
    game_manager: GameManager,

    playing: bool,
    state_text: String,

    search_thread: SearchThread,
    sfx_stream: SFXStream,
}

impl HardfiskurApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            game_manager: GameManager::new(),

            playing: true,
            state_text: "".to_string(),

            search_thread: SearchThread::new(),
            sfx_stream: SFXStream::new(),
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
        if !self.playing {
            return;
        }

        if self.game_manager.push_move(the_move) {
            if the_move.is_capture() {
                self.sfx_stream.play_capture();
            } else {
                self.sfx_stream.play_move();
            }
        }

        self.search_thread.cancel_search();
    }

    fn update_playing(&mut self) {
        let state = self.game_manager.current_board().state();
        self.playing = matches!(state, BoardState::InPlay { .. });
        self.state_text = match state {
            BoardState::InPlay { .. } => "".to_string(),
            BoardState::Draw(DrawReason::FiftyMoveRule) => "Draw by fifty-move rule".to_string(),
            BoardState::Draw(DrawReason::InsufficientMaterial) => {
                "Draw by insufficient material".to_string()
            }
            BoardState::Draw(DrawReason::Stalemate) => "Draw by stalemate".to_string(),
            BoardState::Draw(DrawReason::ThreeFoldRepetition) => {
                "Draw by threefold repetition".to_string()
            }
            BoardState::Win(color) => match color {
                Color::White => "Win by white".to_string(),
                Color::Black => "Win by black".to_string(),
            },
        };
    }
}

impl eframe::App for HardfiskurApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(m) = self.search_thread.try_receive_move() {
            self.make_move(m);
        }

        self.update_playing();

        egui::SidePanel::right("right_panel")
            .resizable(false)
            .min_width(200.0)
            .show(ctx, |ui| {
                ui.label(&format!("State: {}", self.state_text));

                if ui.button("Make move").clicked()
                    || ctx.input(|input| input.key_pressed(Key::Space))
                {
                    if self.playing {
                        self.start_search(ctx);
                    }
                }

                if ui.button("Reset").clicked() {
                    self.game_manager.reset();
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
            ui.label(self.game_manager.current_board().fen());
            ui.label(&format!(
                "{:?}",
                self.game_manager.current_board().zobrist_hash()
            ));
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(
                Layout::centered_and_justified(egui::Direction::LeftToRight),
                |ui| {
                    let input_move = self.game_manager.ui_board(ui);

                    if let Some(m) = input_move {
                        self.make_move(m);
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
