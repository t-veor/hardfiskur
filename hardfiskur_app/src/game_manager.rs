use eframe::egui::{self, Id, Layout, Ui, Vec2};
use egui_extras::{Size, Strip, StripBuilder};
use hardfiskur_core::board::{Board, Color, Move};
use hardfiskur_ui::chess_board::{ChessBoard, ChessBoardData};

#[derive(Debug, Clone)]
struct MoveHistoryItem {
    move_repr: Move,
    san: String,
}

#[derive(Debug, Clone)]
struct GameManagerState {
    current_board: Board,

    starting_fullmoves: u32,
    move_history_position: usize,
    display_board: Board,

    white_first_move: bool,
    move_history: Vec<MoveHistoryItem>,
}

impl GameManagerState {
    fn new(board: Board) -> Self {
        let white_first_move = board.to_move().is_white();
        let starting_fullmoves = board.fullmoves();

        Self {
            current_board: board.clone(),
            starting_fullmoves,
            move_history_position: 0,
            display_board: board,
            white_first_move,
            move_history: Vec::new(),
        }
    }

    fn displaying_current_move(&self) -> bool {
        self.move_history_position >= self.move_history.len()
    }

    fn scroll_forwards(&mut self) {
        self.scroll_to(self.move_history_position + 1);
    }

    fn scroll_backwards(&mut self) {
        self.scroll_to(self.move_history_position.saturating_sub(1));
    }

    fn scroll_to(&mut self, move_history_position: usize) {
        let move_history_position = move_history_position.clamp(0, self.move_history.len());

        while self.move_history_position < move_history_position {
            let item = &self.move_history[self.move_history_position];
            assert!(self.display_board.push_move_repr(item.move_repr));
            self.move_history_position += 1;
        }

        while self.move_history_position > move_history_position {
            self.display_board.pop_move();
            self.move_history_position -= 1;
        }
    }

    fn push_move(&mut self, m: Move) -> bool {
        let san = self.current_board.get_san(m);
        if self.current_board.push_move_repr(m) {
            // Sticky behavior -- if we're currently displaying the latest move,
            // advance it along with the current board.
            if self.displaying_current_move() {
                self.move_history_position += 1;
                assert!(self.display_board.push_move_repr(m));
            }

            self.move_history.push(MoveHistoryItem {
                move_repr: m,
                san: san.map(|s| s.to_string()).unwrap_or("?".to_string()),
            });

            true
        } else {
            false
        }
    }

    fn pop_move(&mut self) {
        self.current_board.pop_move();
        self.move_history.pop();

        while self.move_history_position > self.move_history.len() {
            self.display_board.pop_move();
            self.move_history_position -= 1;
        }
    }

    fn rows(&self) -> Vec<MoveHistoryRow> {
        let mut rows = Vec::new();
        let mut move_iter = self
            .move_history
            .iter()
            .map(|i| i.san.as_str())
            .enumerate()
            .peekable();
        let mut fullmoves = self.starting_fullmoves;

        if !self.white_first_move {
            rows.push(MoveHistoryRow {
                fullmoves,
                white_move: None,
                black_move: move_iter.next(),
            });

            fullmoves += 1;
        }

        while let Some(white_move) = move_iter.next() {
            rows.push(MoveHistoryRow {
                fullmoves,
                white_move: Some(white_move),
                black_move: move_iter.next(),
            });
            fullmoves += 1;
        }

        rows
    }
}

#[derive(Debug, Clone)]
struct MoveHistoryRow<'a> {
    fullmoves: u32,
    white_move: Option<(usize, &'a str)>,
    black_move: Option<(usize, &'a str)>,
}

pub struct GameManager {
    state: GameManagerState,
    chess_ui: ChessBoard,
}

impl GameManager {
    pub fn new() -> Self {
        Self {
            state: GameManagerState::new(Board::starting_position()),
            chess_ui: ChessBoard::new(Id::new("hardfiskur_ui_board")),
        }
    }

    pub fn ui_board(&mut self, ui: &mut Ui) -> Option<Move> {
        let response = self.chess_ui.ui(
            ui,
            ChessBoardData {
                board: &self.state.display_board,
                skip_animation: false,
                can_move: self.state.displaying_current_move(),
                perspective: Color::White,
            },
        );

        if response.egui_response.hovered() {
            ui.input(|state| {
                if state.raw_scroll_delta.y > 0.0 {
                    self.scroll_forwards();
                } else if state.raw_scroll_delta.y < 0.0 {
                    self.scroll_backwards();
                }
            });
        }

        response.input_move
    }

    pub fn ui_move_history(&mut self, ui: &mut Ui) -> Option<usize> {
        let mut scroll_request = None;

        egui::ScrollArea::vertical()
            .stick_to_bottom(true)
            .show(ui, |ui| {
                scroll_request = self.emit_move_history_rows(ui);
            });

        scroll_request
    }

    pub fn scroll_forwards(&mut self) {
        self.state.scroll_forwards();
    }

    pub fn scroll_backwards(&mut self) {
        self.state.scroll_backwards();
    }

    pub fn scroll_to(&mut self, move_history_position: usize) {
        self.state.scroll_to(move_history_position);
    }

    pub fn push_move(&mut self, m: Move) -> bool {
        self.state.push_move(m)
    }

    pub fn pop_move(&mut self) {
        self.state.pop_move();
    }

    pub fn reset(&mut self) {
        self.state = GameManagerState::new(Board::starting_position());
    }

    pub fn current_board(&self) -> &Board {
        &self.state.current_board
    }

    fn emit_move_history_rows(&mut self, ui: &mut Ui) -> Option<usize> {
        let rows = self.state.rows();
        let mut scroll_request = None;

        egui::Grid::new("move_history_grid")
            .striped(true)
            .show(ui, |ui| {
                for row in rows {
                    ui.label(format!("{}.", row.fullmoves));
                    match row.white_move {
                        Some((i, san)) => {
                            if ui.label(san).clicked() {
                                scroll_request = Some(i + 1);
                            };
                        }
                        None => {
                            ui.label("...");
                        }
                    }
                    match row.black_move {
                        Some((i, san)) => {
                            if ui.label(san).clicked() {
                                scroll_request = Some(i + 1);
                            };
                        }
                        None => (),
                    };
                    ui.end_row();
                }
            });

        None
    }
}
