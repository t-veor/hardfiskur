use std::fmt::{Display, Write};

use super::{Board, BoardState, Move, PieceType, Square};

#[derive(Debug, Clone, Copy)]
enum Disambiguator {
    File(u8),
    Rank(u8),
    Square(Square),
}

impl Display for Disambiguator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Disambiguator::File(file) => f.write_char((file + b'a') as char),
            Disambiguator::Rank(rank) => f.write_char((rank + b'1') as char),
            Disambiguator::Square(square) => f.write_fmt(format_args!("{square}")),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct RegularSANRepr {
    piece_type: PieceType,
    disambiguator: Option<Disambiguator>,
    is_capture: bool,
    to_square: Square,
    promotion: Option<PieceType>,
}

impl Display for RegularSANRepr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.piece_type != PieceType::Pawn {
            f.write_char(self.piece_type.as_uppercase_char())?;
        }

        if let Some(disambiguator) = self.disambiguator {
            f.write_fmt(format_args!("{disambiguator}"))?;
        }

        if self.is_capture {
            f.write_char('x')?;
        }

        f.write_fmt(format_args!("{}", self.to_square))?;

        if let Some(promotion) = self.promotion {
            f.write_char('=')?;
            f.write_char(promotion.as_uppercase_char())?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
enum SANRepr {
    Regular(RegularSANRepr),
    Castle { is_long: bool },
}

impl Display for SANRepr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SANRepr::Regular(r) => write!(f, "{r}"),
            SANRepr::Castle { is_long: false } => write!(f, "O-O"),
            SANRepr::Castle { is_long: true } => write!(f, "O-O-O"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SAN {
    repr: SANRepr,
    is_check: bool,
    is_checkmate: bool,
}

impl Display for SAN {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.repr)?;

        if self.is_check {
            write!(f, "+")?;
        }

        if self.is_checkmate {
            write!(f, "#")?;
        }

        Ok(())
    }
}

impl Board {
    pub fn get_san(&self, the_move: Move) -> Option<SAN> {
        let legal_moves = self.legal_moves();
        if !legal_moves.contains(&the_move) {
            return None;
        }

        let repr = if the_move.is_castle() {
            let is_long = the_move.to_square().file() == 2;
            SANRepr::Castle { is_long }
        } else {
            let disambiguator = get_san_disambiguator(the_move, &legal_moves);

            SANRepr::Regular(RegularSANRepr {
                piece_type: the_move.piece().piece_type(),
                disambiguator,
                is_capture: the_move.is_capture(),
                to_square: the_move.to_square(),
                promotion: the_move.promotion().map(|p| p.piece_type()),
            })
        };

        // Make the move. Is it a check or checkmate?
        let mut is_check = false;
        let mut is_checkmate = false;

        let mut board = self.clone();
        board.push_move_unchecked(the_move);

        match board.state() {
            BoardState::InPlay { checkers } => is_check = checkers > 0,
            BoardState::Win(_) => is_checkmate = true,
            _ => (),
        }

        Some(SAN {
            repr,
            is_check,
            is_checkmate,
        })
    }
}

fn get_san_disambiguator(the_move: Move, legal_moves: &[Move]) -> Option<Disambiguator> {
    let mut ambiguous_piece_exists = false;
    let mut same_rank = false;
    let mut same_file = false;

    for m in legal_moves {
        if m.piece() == the_move.piece()
            && m.from_square() != the_move.from_square()
            && m.to_square() == the_move.to_square()
        {
            ambiguous_piece_exists = true;

            same_rank |= m.from_square().rank() == m.to_square().rank();
            same_file |= m.from_square().file() == m.to_square().file();
        }
    }

    if ambiguous_piece_exists {
        if !same_file {
            Some(Disambiguator::File(the_move.from_square().file()))
        } else if !same_rank {
            Some(Disambiguator::Rank(the_move.from_square().rank()))
        } else {
            Some(Disambiguator::Square(the_move.from_square()))
        }
    } else {
        None
    }
}
