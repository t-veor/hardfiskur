use std::io::Cursor;

use eframe::egui::{self, Id, Key, Layout, Vec2};
use hardfiskur_core::board::{Board, BoardState, Color, DrawReason, Move};
use hardfiskur_ui::chess_board::{ChessBoard, ChessBoardData};
use rand::prelude::*;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Source};

struct HardfiskurUI {
    chess_ui: ChessBoard,
    board: Board,

    move_history: Vec<String>,
    just_moved: bool,

    playing: bool,
    state_text: String,

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
            just_moved: false,

            playing: true,
            state_text: "".to_string(),

            _output_stream: stream,
            output_stream_handle: handle,
        }
    }

    fn reset(&mut self) {
        self.board = Board::starting_position();
        self.move_history.clear();
    }

    fn make_move(&mut self, the_move: Move) {
        if !self.playing {
            return;
        }

        let san = self.board.get_san(the_move);

        if self.board.push_move_repr(the_move) {
            let sound_file = if the_move.is_capture() {
                include_bytes!("Capture.ogg").as_slice()
            } else {
                include_bytes!("Move.ogg").as_slice()
            };
            let sound = Decoder::new(Cursor::new(sound_file))
                .unwrap()
                .amplify(0.2)
                .convert_samples();

            self.output_stream_handle.play_raw(sound).unwrap();

            self.move_history.push(match san {
                Some(san) => format!("{san}"),
                None => "?".to_string(),
            });
        }

        println!("{}", self.board);
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

impl eframe::App for HardfiskurUI {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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

        egui::SidePanel::right("right_panel").show(ctx, |ui| {
            ui.label(&format!("State: {}", self.state_text));

            if ui.button("Random move").clicked()
                || ctx.input(|input| input.key_pressed(Key::Space))
            {
                if let Some(the_move) = random_move(&self.board) {
                    self.make_move(the_move);
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

fn random_move(board: &Board) -> Option<Move> {
    let legal_moves = board.legal_moves();
    legal_moves.choose(&mut rand::thread_rng()).copied()
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
