use eframe::egui::{Key, TextEdit, Ui};

#[derive(Debug, Default)]
pub struct FenInput {
    last_known_fen: String,
    prospective_fen: String,
}

impl FenInput {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn show(&mut self, ui: &mut Ui, current_fen: &str) -> Option<String> {
        if self.last_known_fen != current_fen {
            self.last_known_fen = current_fen.to_string();
            self.prospective_fen = current_fen.to_string();
        }

        let response =
            ui.add(TextEdit::singleline(&mut self.prospective_fen).desired_width(f32::INFINITY));

        if response.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter)) {
            Some(self.prospective_fen.clone())
        } else {
            None
        }
    }
}
