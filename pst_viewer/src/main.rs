use eframe::{
    egui::{self, Align2, Color32, Rect, Sense, Vec2},
    epaint::Hsva,
};
use hardfiskur_core::board::{PieceType, Square};
use hardfiskur_engine::evaluation::piece_tables::piece_square_table;

struct PSTViewerUI {
    piece: PieceType,
    endgame_phase: i32,
}

impl PSTViewerUI {
    pub fn new(_cc: &eframe::CreationContext) -> Self {
        Self {
            piece: PieceType::Pawn,
            endgame_phase: 24,
        }
    }
}

fn eval_for_square(piece: PieceType, phase: i32, square: Square) -> i32 {
    let s = piece_square_table(piece, square);
    let (mid, end) = (s.mg(), s.eg());
    (mid * phase + end * (24 - phase)) / 24
}

impl eframe::App for PSTViewerUI {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        egui::Window::new("Options").show(ctx, |ui| {
            egui::ComboBox::from_label("Piece Type")
                .selected_text(format!("{:?}", self.piece))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.piece, PieceType::Pawn, "Pawn");
                    ui.selectable_value(&mut self.piece, PieceType::Knight, "Knight");
                    ui.selectable_value(&mut self.piece, PieceType::Bishop, "Bishop");
                    ui.selectable_value(&mut self.piece, PieceType::Rook, "Rook");
                    ui.selectable_value(&mut self.piece, PieceType::Queen, "Queen");
                    ui.selectable_value(&mut self.piece, PieceType::King, "King");
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
                    .map(|i| eval_for_square(self.piece, self.endgame_phase, i))
                    .collect::<Vec<_>>();
                let max_eval = evals.iter().map(|x| x.abs()).max().unwrap();

                for (i, x) in evals.iter().enumerate() {
                    let square = Square::from_index(i).unwrap();
                    let (rank, file) = (square.rank(), square.file());

                    let center = board_rect.left_top()
                        + Vec2::new(
                            (file as f32 + 0.5) * square_size.x,
                            (8.0 - rank as f32 - 0.5) * square_size.y,
                        );
                    let square_rect = Rect::from_center_size(center, square_size * 0.9);

                    let value = x.abs() as f32 / max_eval as f32;
                    let color = if *x >= 0 {
                        Hsva {
                            h: 0.666,
                            s: 1.0,
                            v: value,
                            a: 1.0,
                        }
                    } else {
                        Hsva {
                            h: 0.0,
                            s: 1.0,
                            v: value,
                            a: 1.0,
                        }
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
