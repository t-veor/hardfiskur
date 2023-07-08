use eframe::egui::{self, Layout, Vec2};
use hardfiskur_core::{
    board::{Bitboard, Piece, PieceType, Square},
    move_gen::{lookups::Lookups, magic::MagicTableEntry},
};
use hardfiskur_ui::base_board::{BaseBoard, BaseBoardData};

struct MagicBitboardViewerUI {
    chess_ui: BaseBoard,
    piece: PieceType,
    square: Square,
    blockers: Bitboard,
    lookups: &'static Lookups,
}

impl MagicBitboardViewerUI {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            chess_ui: Default::default(),
            piece: PieceType::Knight,
            square: Square::new(0, 0).unwrap(),
            blockers: Bitboard::EMPTY,
            lookups: Lookups::get_instance(),
        }
    }
}

impl MagicBitboardViewerUI {
    fn debug_attack_pattern_text(&self) -> (Bitboard, String) {
        match self.piece {
            PieceType::Pawn => unreachable!(),
            PieceType::Knight => self.debug_knight_text(),
            PieceType::Bishop => self.debug_bishop_text(),
            PieceType::Rook => self.debug_rook_text(),
            PieceType::Queen => self.debug_queen_text(),
            PieceType::King => self.debug_king_text(),
        }
    }

    fn debug_knight_text(&self) -> (Bitboard, String) {
        let attack_pattern = self.lookups.get_knight_moves(self.square);
        let debug_string = format!(
            "knight_moves[{}] =\n{attack_pattern:?}",
            self.square.index()
        );

        (attack_pattern, debug_string)
    }

    fn debug_bishop_text(&self) -> (Bitboard, String) {
        let claimed_attack_pattern = self.lookups.get_bishop_attacks(self.blockers, self.square);
        let (calculated_attack_pattern, debug_text) = self.debug_sliding_piece_text(
            "bishop_magics",
            self.square.index(),
            self.lookups.debug_magic_tables().debug_bishop_table(),
        );

        if claimed_attack_pattern != calculated_attack_pattern {
            return (
                claimed_attack_pattern,
                format!(
                    "ERROR: CALCULATED AND RETURNED ATTACK PATTERNS DO NOT MATCH\n\n{debug_text}"
                ),
            );
        }

        (claimed_attack_pattern, debug_text)
    }

    fn debug_rook_text(&self) -> (Bitboard, String) {
        let claimed_attack_pattern = self.lookups.get_rook_attacks(self.blockers, self.square);
        let (calculated_attack_pattern, debug_text) = self.debug_sliding_piece_text(
            "rook_magics",
            self.square.index(),
            self.lookups.debug_magic_tables().debug_rook_table(),
        );

        if claimed_attack_pattern != calculated_attack_pattern {
            return (
                claimed_attack_pattern,
                format!(
                    "ERROR: CALCULATED AND RETURNED ATTACK PATTERNS DO NOT MATCH\n\n{debug_text}"
                ),
            );
        }

        (claimed_attack_pattern, debug_text)
    }

    fn debug_queen_text(&self) -> (Bitboard, String) {
        let claimed_attack_pattern = self.lookups.get_queen_attacks(self.blockers, self.square);

        let (calculated_bishop_pattern, _) = self.debug_sliding_piece_text(
            "bishop_magics",
            self.square.index(),
            self.lookups.debug_magic_tables().debug_bishop_table(),
        );
        let (calculated_rook_pattern, _) = self.debug_sliding_piece_text(
            "rook_magics",
            self.square.index(),
            self.lookups.debug_magic_tables().debug_rook_table(),
        );

        let mut debug_text = String::new();
        debug_text.push_str(&format!(
            "lookup_bishop_magic({}) =\n{:?}\n\n",
            self.square.index(),
            calculated_bishop_pattern,
        ));
        debug_text.push_str(&format!(
            "lookup_rook_magic({}) =\n{:?}\n\n",
            self.square.index(),
            calculated_rook_pattern,
        ));

        let calculated_attack_pattern = calculated_bishop_pattern | calculated_rook_pattern;

        debug_text.push_str(&format!(
            "bishop | rook =\n{:?}\n\n",
            calculated_attack_pattern
        ));

        if claimed_attack_pattern != calculated_attack_pattern {
            return (
                claimed_attack_pattern,
                format!(
                    "ERROR: CALCULATED AND RETURNED ATTACK PATTERNS DO NOT MATCH\n\n{debug_text}"
                ),
            );
        }

        (claimed_attack_pattern, debug_text)
    }

    fn debug_king_text(&self) -> (Bitboard, String) {
        let attack_pattern = self.lookups.get_king_moves(self.square);
        let debug_string = format!("king_moves[{}] =\n{attack_pattern:?}", self.square.index());

        (attack_pattern, debug_string)
    }

    fn debug_sliding_piece_text(
        &self,
        table_name: &str,
        idx: usize,
        table: &[MagicTableEntry<'_>; 64],
    ) -> (Bitboard, String) {
        let mut debug_string = String::new();

        let entry = &table[idx];

        debug_string.push_str(&format!("{table_name}[{idx}].mask =\n{:?}\n\n", entry.mask));
        debug_string.push_str(&format!(
            "blockers & mask =\n{:?}\n\n",
            self.blockers & entry.mask
        ));
        debug_string.push_str(&format!(
            "{table_name}[{idx}].magic =\n0x{:016x}\n\n",
            entry.magic,
        ));
        debug_string.push_str(&format!("{table_name}[{idx}].shift =\n{}\n\n", entry.shift,));

        let unshifted_index = (self.blockers & entry.mask).0.wrapping_mul(entry.magic);
        let shifted_index = unshifted_index >> entry.shift;

        debug_string.push_str(&format!(
            "(blockers & mask) * magic >> shift =\n{shifted_index}\n\n"
        ));

        let found_attacks = entry.attack_table[shifted_index as usize];

        debug_string.push_str(&format!(
            "{table_name}[{idx}].attacks[{shifted_index}] =\n{:?}\n\n",
            found_attacks
        ));

        (found_attacks, debug_string)
    }
}

impl eframe::App for MagicBitboardViewerUI {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut attack_pattern = Bitboard::EMPTY;

        egui::SidePanel::left("options").show(ctx, |ui| {
            ui.heading("Bitboard Attack Viewer");

            ui.separator();

            ui.label("Utility to view and debug bitboard attack patterns generated by each piece.");
            ui.label("Drag the white attacking piece around to view different patterns.");
            ui.label("Click on the board to add blockers.");

            ui.separator();

            egui::ComboBox::from_label("Piece Type")
                .selected_text(format!("{:?}", self.piece))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.piece, PieceType::Knight, "Knight");
                    ui.selectable_value(&mut self.piece, PieceType::Bishop, "Bishop");
                    ui.selectable_value(&mut self.piece, PieceType::Rook, "Rook");
                    ui.selectable_value(&mut self.piece, PieceType::Queen, "Queen");
                    ui.selectable_value(&mut self.piece, PieceType::King, "King");
                });

            let (bitboard, debug_string) = self.debug_attack_pattern_text();
            attack_pattern = bitboard;

            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.monospace(debug_string);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(
                Layout::centered_and_justified(egui::Direction::LeftToRight),
                |ui| {
                    let mut board = [None; 64];
                    for (i, piece) in board.iter_mut().enumerate() {
                        if Square::from_index_unchecked(i) == self.square {
                            *piece = Some(Piece::white(self.piece));
                        } else if self.blockers.get(Square::from_index_unchecked(i)) {
                            *piece = Some(Piece::BLACK_PAWN);
                        }
                    }

                    let data = BaseBoardData {
                        pieces: &board[..],
                        display_bitboard: attack_pattern,
                        drag_mask: Bitboard::from_square(self.square),
                        ..Default::default()
                    };

                    let response = self.chess_ui.ui(ui, data);

                    if let Some((start, end)) = response.dropped {
                        if start == self.square {
                            self.square = end;
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
        "Bitboard Attack Viewer",
        eframe::NativeOptions {
            initial_window_size: Some(Vec2::new(1024.0, 768.0)),
            ..Default::default()
        },
        Box::new(|cc| Box::new(MagicBitboardViewerUI::new(cc))),
    )
}
