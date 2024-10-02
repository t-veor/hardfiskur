use std::iter::repeat;

use egui::{Id, Ui};
use hardfiskur_core::board::{Piece, Square};

#[derive(Debug, Clone, Copy)]
pub enum AnimatedPieceState {
    Static(Square),
    Moving {
        from: Square,
        to: Square,
        fraction: f32,
    },
    Emphemeral {
        on: Square,
        fraction: f32,
    },
}

#[derive(Debug, Clone, Copy)]
enum PieceState {
    Static,
    Moving { from: Square },
}

#[derive(Debug)]
pub struct SpriteState {
    // Pieces are always stored at their final correct positions, but moving
    // pieces store where they're moving from.
    pieces: [Option<(Piece, PieceState)>; 64],

    // Ephermeral pieces, drawn for a little bit before disappearing.
    being_captured_pieces: Vec<(Square, Piece)>,

    animation_value: f32,
    animation_id: Id,
    just_started_anim: bool,
}

impl SpriteState {
    pub fn new(id: Id) -> Self {
        Self {
            pieces: [None; 64],
            being_captured_pieces: Vec::new(),
            animation_value: 1.0,
            animation_id: id.with("hardfiskur__base_board_animations"),
            just_started_anim: false,
        }
    }

    fn replace_pieces(&mut self, incoming_pieces: &[Option<Piece>]) {
        self.clear_current_animation();

        for (piece, incoming_piece) in self
            .pieces
            .iter_mut()
            .zip(incoming_pieces.iter().copied().chain(repeat(None)))
        {
            *piece = incoming_piece.map(|p| (p, PieceState::Static))
        }
    }

    pub fn merge_pieces(&mut self, ui: &mut Ui, incoming_pieces: &[Option<Piece>], replace: bool) {
        if replace {
            return self.replace_pieces(incoming_pieces);
        }

        let mut disappearing_pieces = Vec::new();
        let mut new_pieces = Vec::new();
        let mut static_pieces = Vec::new();

        for square in Square::all() {
            let existing_piece = self.pieces[square.index()].map(|(piece, _)| piece);
            let incoming_piece = incoming_pieces.get(square.index()).and_then(|&x| x);

            if existing_piece != incoming_piece {
                if let Some(existing_piece) = existing_piece {
                    disappearing_pieces.push((square, existing_piece));
                }
                if let Some(incoming_piece) = incoming_piece {
                    new_pieces.push((square, incoming_piece));
                }
            } else {
                static_pieces.push(square);
            }
        }

        if disappearing_pieces.is_empty() && new_pieces.is_empty() {
            return;
        }

        // Find pieces that should be moved (in disappearing pieces in one place
        // but in moving pieces in another)
        let mut moving_pieces = Vec::new();
        disappearing_pieces.retain(|&(from_square, piece)| {
            match new_pieces
                .iter()
                .enumerate()
                .filter(|(_, &(_, new_piece))| piece == new_piece)
                .min_by_key(|(_, &(square, _))| from_square.manhattan_distance(square))
            {
                Some((idx, _)) => {
                    let (to_square, _) = new_pieces.swap_remove(idx);
                    moving_pieces.push(((from_square, to_square), piece));
                    false
                }
                None => true,
            }
        });

        // Apply the updates
        self.clear_current_animation();

        let mut next_pieces = [None; 64];
        for square in static_pieces {
            next_pieces[square.index()] = self.pieces[square.index()];
        }

        for (square, disappearing_piece) in disappearing_pieces {
            self.being_captured_pieces
                .push((square, disappearing_piece));
        }

        for (square, new_piece) in new_pieces {
            next_pieces[square.index()] = Some((new_piece, PieceState::Static));
        }

        for ((from_square, to_square), moving_piece) in moving_pieces {
            next_pieces[to_square.index()] =
                Some((moving_piece, PieceState::Moving { from: from_square }));
        }

        self.pieces = next_pieces;

        // Reset animation
        self.animation_id = self.animation_id.with("next");
        self.animation_value = ui.ctx().animate_bool(self.animation_id, false);
        self.just_started_anim = true;
    }

    pub fn update(&mut self, ui: &mut Ui) {
        if self.just_started_anim {
            self.just_started_anim = false;
        } else {
            self.animation_value = ui.ctx().animate_bool(self.animation_id, true);
        }
    }

    fn clear_current_animation(&mut self) {
        self.being_captured_pieces.clear();
        for i in self.pieces.iter_mut() {
            let Some((_, piece_state)) = i else { continue };
            if matches!(piece_state, PieceState::Moving { .. }) {
                *piece_state = PieceState::Static;
            }
        }
    }

    pub fn get_pieces(&self) -> impl Iterator<Item = (Piece, AnimatedPieceState)> + '_ {
        self.being_captured_pieces
            .iter()
            .map(|&(square, piece)| {
                (
                    piece,
                    AnimatedPieceState::Emphemeral {
                        on: square,
                        fraction: self.animation_value,
                    },
                )
            })
            .chain(self.pieces.iter().enumerate().filter_map(|(idx, &piece)| {
                let (piece, piece_state) = piece?;
                let square = Square::from_index(idx)?;

                let animated_piece_state = match piece_state {
                    PieceState::Static => AnimatedPieceState::Static(square),
                    PieceState::Moving { from } => AnimatedPieceState::Moving {
                        from,
                        to: square,
                        fraction: self.animation_value,
                    },
                };

                Some((piece, animated_piece_state))
            }))
    }
}
