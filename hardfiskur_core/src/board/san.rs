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

/// Representation of a move in [Standard Algebraic Notation
/// (SAN)](https://en.wikipedia.org/wiki/Algebraic_notation_(chess)). Checks and
/// checkmates are indicated with a `+` and `#` suffix respectively.
///
/// Since the SAN of a move depends on the current board state, this struct must
/// be obtained from a [`Board`] in the correct state prior to the move being
/// made.
///
/// Use the [`Display`] implementation to get the string representation of a
/// [`SAN`] object.
///
/// # Example
/// ```
/// # use hardfiskur_core::board::{Board, SAN, Square};
/// fn push_move_and_get_san(board: &mut Board, from: Square, to: Square) -> SAN {
///     let the_move = board.get_move(from, to, None).unwrap(); // Assuming no promotions
///     // Get the move SAN before making the move
///     let san = board.get_san(the_move).unwrap();
///     assert!(board.push_move_repr(the_move));
///     san
/// }
///
/// let mut board = Board::starting_position();
///
/// let e2e4 = push_move_and_get_san(&mut board, Square::E2, Square::E4);
/// assert_eq!(e2e4.to_string(), "e4");
///
/// let d7d5 = push_move_and_get_san(&mut board, Square::D7, Square::D5);
/// assert_eq!(d7d5.to_string(), "d5");
///
/// let exd5 = push_move_and_get_san(&mut board, Square::E4, Square::D5);
/// assert_eq!(exd5.to_string(), "exd5");
///
/// let qxd5 = push_move_and_get_san(&mut board, Square::D8, Square::D5);
/// assert_eq!(qxd5.to_string(), "Qxd5");
/// ```
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
    if the_move.piece().is_pawn() && the_move.is_capture() {
        // Pawn captures always need to have the origin file.
        // (Two pawns on the same rank can't capture the same square.)
        return Some(Disambiguator::File(the_move.from_square().file()));
    }

    let mut ambiguous_piece_exists = false;
    let mut same_rank = false;
    let mut same_file = false;

    for m in legal_moves {
        if m.piece() == the_move.piece()
            && m.from_square() != the_move.from_square()
            && m.to_square() == the_move.to_square()
        {
            ambiguous_piece_exists = true;

            same_rank |= m.from_square().rank() == the_move.from_square().rank();
            same_file |= m.from_square().file() == the_move.from_square().file();
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

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    fn push_move_and_get_san(
        board: &mut Board,
        from: Square,
        to: Square,
        promotion: Option<PieceType>,
    ) -> SAN {
        let the_move = board.get_move(from, to, promotion).unwrap();
        let san = board.get_san(the_move).unwrap();
        assert!(board.push_move_repr(the_move));
        san
    }

    #[test]
    fn pawn_push() {
        let mut board = Board::starting_position();

        let e4 = push_move_and_get_san(&mut board, Square::E2, Square::E4, None);
        assert_eq!(e4.to_string(), "e4");

        let c6 = push_move_and_get_san(&mut board, Square::C7, Square::C6, None);
        assert_eq!(c6.to_string(), "c6");
    }

    #[test]
    fn pawn_captures() {
        let mut board = Board::try_parse_fen("4k3/8/8/3p4/1p2P3/8/PK6/8 w - - 0 1").unwrap();

        let exd5 = push_move_and_get_san(&mut board, Square::E4, Square::D5, None);
        assert_eq!(exd5.to_string(), "exd5");

        board.push_move(Square::E8, Square::D7, None).unwrap();
        board.push_move(Square::A2, Square::A4, None).unwrap();

        // en passant
        let bxa3_check = push_move_and_get_san(&mut board, Square::B4, Square::A3, None);
        assert_eq!(bxa3_check.to_string(), "bxa3+");
    }

    #[test]
    fn pawn_promotions() {
        let mut board = Board::try_parse_fen("1k6/3P4/8/8/8/8/2p5/3R1K2 w - - 0 1").unwrap();

        let d8_promote_knight =
            push_move_and_get_san(&mut board, Square::D7, Square::D8, Some(PieceType::Knight));
        assert_eq!(d8_promote_knight.to_string(), "d8=N");

        let c_captures_d1_promote_queen_check =
            push_move_and_get_san(&mut board, Square::C2, Square::D1, Some(PieceType::Queen));
        assert_eq!(c_captures_d1_promote_queen_check.to_string(), "cxd1=Q+");
    }

    #[test]
    fn piece_moves() {
        let mut board = Board::try_parse_fen("3qkb2/8/8/8/8/8/4P3/3RK1N1 w - - 0 1").unwrap();

        let nf3 = push_move_and_get_san(&mut board, Square::G1, Square::F3, None);
        assert_eq!(nf3.to_string(), "Nf3");

        let bb4_check = push_move_and_get_san(&mut board, Square::F8, Square::B4, None);
        assert_eq!(bb4_check.to_string(), "Bb4+");

        let rd2 = push_move_and_get_san(&mut board, Square::D1, Square::D2, None);
        assert_eq!(rd2.to_string(), "Rd2");

        let qf6 = push_move_and_get_san(&mut board, Square::D8, Square::F6, None);
        assert_eq!(qf6.to_string(), "Qf6");
    }

    #[test]
    fn castles() {
        let mut board = Board::try_parse_fen("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1").unwrap();

        let o_o = push_move_and_get_san(&mut board, Square::E1, Square::G1, None);
        assert_eq!(o_o.to_string(), "O-O");

        let o_o_o = push_move_and_get_san(&mut board, Square::E8, Square::C8, None);
        assert_eq!(o_o_o.to_string(), "O-O-O");
    }

    #[test]
    fn castle_check() {
        let mut board = Board::try_parse_fen("5k2/8/8/8/8/8/8/4K2R w K - 0 1").unwrap();

        let o_o_check = push_move_and_get_san(&mut board, Square::E1, Square::G1, None);
        assert_eq!(o_o_check.to_string(), "O-O+");
    }

    #[test]
    fn castle_checkmate() {
        let mut board = Board::try_parse_fen("3k4/2p5/B7/8/4R3/8/8/R3K3 w Q - 0 1").unwrap();

        let o_o_o_checkmate = push_move_and_get_san(&mut board, Square::E1, Square::C1, None);
        assert_eq!(o_o_o_checkmate.to_string(), "O-O-O#");
    }

    #[test]
    fn file_disambiguation() {
        let mut board = Board::try_parse_fen("3k4/8/8/8/5n2/4n3/8/4K3 b - - 0 1").unwrap();

        let nfd5 = push_move_and_get_san(&mut board, Square::F4, Square::D5, None);
        assert_eq!(nfd5.to_string(), "Nfd5");
    }

    #[test]
    fn rank_disambiguation() {
        let mut board = Board::try_parse_fen("8/8/6R1/k7/8/6R1/8/4K3 w - - 0 1").unwrap();

        let r3g5_check = push_move_and_get_san(&mut board, Square::G3, Square::G5, None);
        assert_eq!(r3g5_check.to_string(), "R3g5+");
    }

    #[test]
    fn rank_and_file_disambiguation() {
        let mut board = Board::try_parse_fen("4k3/6Q1/8/4p3/8/2Q3Q1/8/3RK3 w Q - 0 1").unwrap();

        let qg3_capture_e5_checkmate =
            push_move_and_get_san(&mut board, Square::G3, Square::E5, None);
        assert_eq!(qg3_capture_e5_checkmate.to_string(), "Qg3xe5#")
    }
}
