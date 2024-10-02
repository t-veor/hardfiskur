mod search_thread;
mod sfx_stream;

use eframe::egui::{self, Id, Key, Layout, Vec2};
use hardfiskur_core::board::{Board, BoardState, Color, DrawReason, Move};
use hardfiskur_ui::chess_board::{ChessBoard, ChessBoardData};

use search_thread::SearchThread;
use sfx_stream::SFXStream;

struct HardfiskurApp {
    chess_ui: ChessBoard,
    board: Board,

    move_history: Vec<String>,
    just_moved: bool,

    playing: bool,
    state_text: String,

    search_thread: SearchThread,
    sfx_stream: SFXStream,
}

impl HardfiskurApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            chess_ui: ChessBoard::new(Id::new("hardfiskur_ui_board")),
            board: Board::starting_position(),

            move_history: vec![],
            just_moved: false,

            playing: true,
            state_text: "".to_string(),

            search_thread: SearchThread::new(),
            sfx_stream: SFXStream::new(),
        }
    }

    fn reset(&mut self) {
        self.board = Board::starting_position();
        self.move_history.clear();
    }

    fn start_search(&mut self, ctx: &egui::Context) {
        if !self.search_thread.searching() {
            let ctx = ctx.clone();
            self.search_thread
                .send_search_request(&self.board, move || {
                    ctx.request_repaint();
                });
        }
    }

    fn make_move(&mut self, the_move: Move) {
        if !self.playing {
            return;
        }

        let san = self.board.get_san(the_move);

        if self.board.push_move_repr(the_move) {
            if the_move.is_capture() {
                self.sfx_stream.play_capture();
            } else {
                self.sfx_stream.play_move();
            };

            self.move_history.push(match san {
                Some(san) => format!("{san}"),
                None => "?".to_string(),
            });
        }

        self.search_thread.cancel_search();
    }

    fn undo_move(&mut self) {
        if self.board.pop_move().is_some() {
            self.playing = true;
            self.move_history.pop();
        }
    }

    fn update_playing(&mut self) {
        let state = self.board.state();
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

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(
                Layout::centered_and_justified(egui::Direction::LeftToRight),
                |ui| {
                    let response = self.chess_ui.ui(
                        ui,
                        ChessBoardData {
                            board: &self.board,
                            skip_animation: self.just_moved,
                            can_move: true,
                            perspective: Color::White,
                        },
                    );

                    self.just_moved = false;

                    if let Some(m) = response.input_move {
                        self.make_move(m);
                        self.just_moved = true;
                    }
                },
            );
        });

        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.label(self.board.fen());
            ui.label(&format!("{:?}", self.board.zobrist_hash()));
        });

        egui::SidePanel::right("right_panel").show(ctx, |ui| {
            ui.label(&format!("State: {}", self.state_text));

            if ui.button("Make move").clicked() || ctx.input(|input| input.key_pressed(Key::Space))
            {
                if self.playing {
                    self.start_search(ctx);
                }
            }

            if ui.button("Reset").clicked() {
                self.reset();
            }

            if ui.button("Undo move").clicked() {
                self.undo_move();
            }

            egui::ScrollArea::vertical()
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    egui::Grid::new("moves").striped(true).show(ui, |ui| {
                        for (i, m) in self.move_history.iter().enumerate() {
                            if i % 2 == 0 {
                                ui.label(format!("{}.", (i / 2) + 1));
                            }
                            ui.label(m);
                            if i % 2 == 1 {
                                ui.end_row();
                            }
                        }
                    });
                });
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
