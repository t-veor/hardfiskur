use eframe::{
    egui::{self, Align2, Color32, Rect, Sense, Vec2},
    epaint::Hsva,
};
use hardfiskur_core::board::Square;
use hardfiskur_engine::evaluation::{
    parameters::{BISHOP_PST, KING_PST, KNIGHT_PST, PASSED_PAWNS, PAWN_PST, QUEEN_PST, ROOK_PST},
    phase::Phase,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TableType {
    PawnPST,
    KnightPST,
    BishopPST,
    RookPST,
    QueenPST,
    KingPST,
    PassedPawns,
}

impl TableType {
    const ALL: &[Self] = &[
        Self::PawnPST,
        Self::KnightPST,
        Self::BishopPST,
        Self::RookPST,
        Self::QueenPST,
        Self::KingPST,
        Self::PassedPawns,
    ];
}

struct PSTViewerUI {
    table_type: TableType,
    endgame_phase: i32,
}

impl PSTViewerUI {
    pub fn new(_cc: &eframe::CreationContext) -> Self {
        Self {
            table_type: TableType::PawnPST,
            endgame_phase: 24,
        }
    }
}

fn table_value(table_type: TableType, phase: i32, square: Square) -> i32 {
    let table = match table_type {
        TableType::PawnPST => &PAWN_PST,
        TableType::KnightPST => &KNIGHT_PST,
        TableType::BishopPST => &BISHOP_PST,
        TableType::RookPST => &ROOK_PST,
        TableType::QueenPST => &QUEEN_PST,
        TableType::KingPST => &KING_PST,
        TableType::PassedPawns => &PASSED_PAWNS,
    };

    let phase = Phase(phase);
    phase.taper_packed(table[square.index()])
}

impl eframe::App for PSTViewerUI {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        egui::Window::new("Options").show(ctx, |ui| {
            egui::ComboBox::from_label("Table")
                .selected_text(format!("{:?}", self.table_type))
                .show_ui(ui, |ui| {
                    for &table_type in TableType::ALL {
                        ui.selectable_value(
                            &mut self.table_type,
                            table_type,
                            format!("{table_type:?}"),
                        );
                    }
                });

            ui.label("Endgame Phase");
            ui.add(egui::Slider::new(&mut self.endgame_phase, 0..=24));
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.centered_and_justified(|ui| {
                let space = ui.available_size();
                let board_size = Vec2::splat(space.x.min(space.y) - 20.0);
                let square_size = Vec2::splat(board_size.x / 8.0);
                let (response, painter) = ui.allocate_painter(board_size, Sense::hover());

                let board_rect = Rect::from_center_size(response.rect.center(), board_size);

                let evals = Square::all()
                    .map(|i| table_value(self.table_type, self.endgame_phase, i))
                    .collect::<Vec<_>>();
                let avg = evals.iter().sum::<i32>() as f32 / evals.len() as f32;
                let max_eval = *evals.iter().max().unwrap();
                let min_eval = *evals.iter().min().unwrap();

                let scale_factor = (avg - max_eval as f32)
                    .abs()
                    .max(avg - min_eval as f32)
                    .abs()
                    .max(1.0);

                for (i, x) in evals.iter().enumerate() {
                    let square = Square::from_index(i).unwrap();
                    let (rank, file) = (square.rank(), square.file());

                    let center = board_rect.left_top()
                        + Vec2::new(
                            (file as f32 + 0.5) * square_size.x,
                            (rank as f32 + 0.5) * square_size.y,
                        );
                    let square_rect = Rect::from_center_size(center, square_size * 0.9);

                    let value = (*x as f32 - avg) / scale_factor;
                    let color = Hsva {
                        h: if value >= 0.0 { 0.666 } else { 0.0 },
                        s: 1.0,
                        v: value.abs(),
                        a: 1.0,
                    };

                    painter.rect_filled(square_rect, 0.0, color);
                    painter.text(
                        center,
                        Align2::CENTER_CENTER,
                        format!("{x}"),
                        Default::default(),
                        Color32::WHITE,
                    );
                }
            });
        });
    }
}

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "PST Viewer",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size(Vec2::new(1024.0, 768.0)),
            ..Default::default()
        },
        Box::new(|cc| Ok(Box::new(PSTViewerUI::new(cc)))),
    )
}
