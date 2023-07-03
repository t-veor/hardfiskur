use eframe::egui::{self, Layout, Vec2};
use hardfiskur_core::{
    board::{bitboard::Bitboard, Color, Piece, PieceType, Square},
    move_gen::lookups::Lookups,
};
use hardfiskur_ui::ui::{ChessUI, ChessUIData};

struct MagicBitboardViewerUI {
    chess_ui: ChessUI,
    piece: PieceType,
    square: Square,
    blockers: Bitboard,
}

impl MagicBitboardViewerUI {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            chess_ui: Default::default(),
            piece: PieceType::Rook,
            square: Square::WHITE_KINGSIDE_ROOK,
            blockers: Bitboard::EMPTY,
        }
    }
}

impl eframe::App for MagicBitboardViewerUI {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left("options").show(ctx, |ui| {
            egui::ComboBox::from_label("Piece Type")
                .selected_text(format!("{:?}", self.piece))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.piece, PieceType::Knight, "Knight");
                    ui.selectable_value(&mut self.piece, PieceType::Bishop, "Bishop");
                    ui.selectable_value(&mut self.piece, PieceType::Rook, "Rook");
                    ui.selectable_value(&mut self.piece, PieceType::Queen, "Queen");
                    ui.selectable_value(&mut self.piece, PieceType::King, "King");
                })
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(
                Layout::centered_and_justified(egui::Direction::LeftToRight),
                |ui| {
                    let lookups = Lookups::get_instance();

                    let bitboard = match self.piece {
                        PieceType::Pawn => Bitboard::EMPTY,
                        PieceType::Knight => lookups.get_knight_moves(self.square),
                        PieceType::Bishop => lookups.get_bishop_attacks(self.blockers, self.square),
                        PieceType::Rook => lookups.get_rook_attacks(self.blockers, self.square),
                        PieceType::Queen => lookups.get_queen_attacks(self.blockers, self.square),
                        PieceType::King => lookups.get_king_moves(self.square),
                    };

                    let mut board = [None; 64];
                    for (i, piece) in board.iter_mut().enumerate() {
                        if Square::from_index_unchecked(i) == self.square {
                            *piece = Some(Piece::new(Color::White, self.piece));
                        } else if self.blockers.get(Square::from_index_unchecked(i)) {
                            *piece = Some(Piece::new(Color::Black, PieceType::Pawn));
                        }
                    }

                    let data = ChessUIData {
                        pieces: &board[..],
                        display_bitboard: Some(bitboard),
                        drag_mask: Bitboard::from_square(self.square),
                        ..Default::default()
                    };

                    let response = self.chess_ui.ui(ui, data);

                    if let Some((start, end)) = response.dropped {
                        if start == self.square {
                            self.square = end;
                            // self.blockers = self.blockers.without(Bitboard::from_square(end));
                        }
                    }

                    if let Some(square) = response.clicked_square {
                        if square != self.square {
                            self.blockers ^= Bitboard::from_square(square);
                        }
                    }
                },
            )
        });
    }
}

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "Magic Bitboard Viewer",
        eframe::NativeOptions {
            initial_window_size: Some(Vec2::new(1600.0, 900.0)),
            ..Default::default()
        },
        Box::new(|cc| Box::new(MagicBitboardViewerUI::new(cc))),
    )
}