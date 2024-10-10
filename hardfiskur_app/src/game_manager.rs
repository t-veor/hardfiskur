use core::f32;
use std::time::{Duration, Instant};

use eframe::egui::{self, Align, Id, Layout, Sense, Ui};
use egui_extras::{Column, TableBuilder, TableRow};
use hardfiskur_core::board::{Board, BoardState, Color, DrawReason, Move};
use hardfiskur_ui::chess_board::ChessBoardUI;

const SOFT_SCROLL_DELAY: Duration = Duration::from_millis(300);
const SCROLL_OVERRIDE_MAGNITUDE: f32 = 3.5;

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

    fn is_displaying_latest_move(&self) -> bool {
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
            if self.is_displaying_latest_move() {
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

    fn is_last_move(&self, idx: usize) -> bool {
        idx + 1 == self.move_history_position
    }

    fn current_display_move(&self) -> Option<&MoveHistoryItem> {
        if self.move_history_position == 0 {
            None
        } else {
            self.move_history.get(self.move_history_position - 1)
        }
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
    chess_ui: ChessBoardUI,

    last_scroll_event: Instant,
}

impl GameManager {
    pub fn new() -> Self {
        Self {
            state: GameManagerState::new(Board::starting_position()),
            chess_ui: ChessBoardUI::new(Id::new("hardfiskur_ui_board")),

            last_scroll_event: Instant::now(),
        }
    }

    pub fn playing(&self) -> bool {
        matches!(self.state.current_board.state(), BoardState::InPlay { .. })
    }

    pub fn ui_board(&mut self, ui: &mut Ui) -> Option<Move> {
        let game_state = self.state.current_board.state();
        let playing = matches!(game_state, BoardState::InPlay { .. });
        let game_state_text = match game_state {
            BoardState::InPlay { .. } => "",
            BoardState::Draw(DrawReason::FiftyMoveRule) => "Draw by fifty-move rule",
            BoardState::Draw(DrawReason::InsufficientMaterial) => "Draw by insufficient material",
            BoardState::Draw(DrawReason::Stalemate) => "Draw by stalemate",
            BoardState::Draw(DrawReason::ThreeFoldRepetition) => "Draw by threefold repetition",
            BoardState::Win(color) => match color {
                Color::White => "White wins by checkmate",
                Color::Black => "Black wins by checkmate",
            },
        };

        let mut input_move = None;

        ui.vertical_centered(|ui| {
            ui.heading(if playing { "" } else { "Game Over" });
            ui.label(game_state_text);

            ui.centered_and_justified(|ui| {
                let mut props = ChessBoardUI::props(&self.state.display_board)
                    .can_move(playing && self.state.is_displaying_latest_move())
                    .fade_out_board(!self.state.is_displaying_latest_move());

                if let Some(item) = self.state.current_display_move() {
                    props = props
                        .show_last_move(item.move_repr.from_square(), item.move_repr.to_square());
                }

                let response = self.chess_ui.ui(ui, props);

                if response.egui_response.hovered() {
                    ui.input(|state| {
                        // This seems to be a good compromise for scrolling on both
                        // notched scroll wheels and touchscreens.
                        // Normally, notched scroll wheels will produce a scroll
                        // magnitude greater than SCROLL_OVERRIDE_MAGNITUDE on a single
                        // frame, so we immediately scroll when this happens.
                        // Touchscreens provide a smaller continuous scroll delta so if
                        // the user is scrolling slowly, then a scroll will be triggered
                        // only with a interval of SOFT_SCROLL_DELAY, allowing fine
                        // control. However, if they scroll with a quick motion then
                        // this will trigger a a fast scroll to the beginning/end of the
                        // move list.
                        let scroll_magnitude = state.raw_scroll_delta.y.abs();
                        if scroll_magnitude >= SCROLL_OVERRIDE_MAGNITUDE
                            || self.last_scroll_event.elapsed() >= SOFT_SCROLL_DELAY
                        {
                            if state.raw_scroll_delta.y > 0.0 {
                                self.scroll_forwards();
                            } else if state.raw_scroll_delta.y < 0.0 {
                                self.scroll_backwards();
                            }
                            self.last_scroll_event = Instant::now();
                        }
                    });
                }

                input_move = response.input_move;
            })
        });

        input_move
    }

    pub fn ui_move_history(&mut self, ui: &mut Ui) -> Option<usize> {
        self.emit_move_history_rows(ui)
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
        self.reset_to(Board::starting_position());
    }

    pub fn reset_to(&mut self, board: Board) {
        self.state = GameManagerState::new(board);
    }

    pub fn current_board(&self) -> &Board {
        &self.state.current_board
    }

    fn emit_move_history_rows(&mut self, ui: &mut Ui) -> Option<usize> {
        let rows = self.state.rows();
        let mut scroll_request = None;

        let text_height = egui::TextStyle::Body
            .resolve(ui.style())
            .size
            .max(ui.spacing().interact_size.y);

        ui.style_mut().interaction.selectable_labels = false;

        TableBuilder::new(ui)
            .column(Column::auto().at_least(24.0))
            .column(Column::remainder())
            .column(Column::remainder())
            .cell_layout(Layout::left_to_right(Align::Center))
            .sense(Sense::click())
            .stick_to_bottom(true)
            .striped(true)
            .body(|body| {
                body.rows(text_height, rows.len(), |mut r| {
                    let row = &rows[r.index()];

                    let mut selectable_cell = |r: &mut TableRow, (i, san): (usize, &str)| {
                        r.set_selected(self.state.is_last_move(i));
                        let response = r
                            .col(|ui| {
                                ui.label(san);
                            })
                            .1;

                        if response.clicked() {
                            scroll_request = Some(i + 1);
                        }
                    };

                    r.set_selected(false);

                    r.col(|ui| {
                        ui.label(format!("{}.", row.fullmoves));
                    });

                    match row.white_move {
                        Some(m) => selectable_cell(&mut r, m),
                        None => {
                            r.col(|ui| {
                                ui.label("...");
                            });
                        }
                    }
                    if let Some(m) = row.black_move {
                        selectable_cell(&mut r, m);
                    }
                });
            });

        scroll_request
    }
}
