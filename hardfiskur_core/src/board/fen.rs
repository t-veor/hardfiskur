use thiserror::Error;

use super::{Board, Castling, Color, Piece, Square};

/// Error type returned by [`Board::try_parse_fen`].
#[derive(Error, Debug)]
pub enum FenParseError {
    /// An incorrect number of fields were found in the FEN string.
    #[error("Expected 6 fields in FEN but found {actual}")]
    IncorrectFieldCount { actual: usize },

    /// An unknown piece code was encountered.
    #[error("Unknown piece `{piece}` encountered")]
    UnknownPiece { piece: char },

    /// An incorrect number of ranks were found in the FEN string.
    #[error("Expected 8 ranks in board string but found {actual}")]
    IncorrectRankCount { actual: usize },

    /// For at least one rank, the total number of squares represented does not
    /// add up to 8.
    #[error("Expected 8 squares on rank {rank} but found {actual}")]
    IncorrectFileCount { rank: u8, actual: usize },

    /// A current player other than `w` or `b` was provided.
    #[error("Expected `w` or `b` as the current player")]
    InvalidCurrentPlayer,

    /// A castling state which is not `-` or some combination of the characters
    /// `K`, `Q`, `k`, and `q` was found.
    #[error("Expected `-` or some combination of `KQkq` as the castling state")]
    InvalidCastling,

    /// An en passant state which is not `-` or the name of a square in
    /// algebraic notation was found.
    #[error("Expected `-` or a valid square in algebraic notation as the en passant state")]
    InvalidEnPassant,

    /// An invalid or negative integer was found for the half move clock.
    #[error("Expected a non-negative integer for the half move clock")]
    InvalidHalfMoveClock,

    /// An invalid or non-positive integer was found for the move count.
    #[error("Expected a positive integer for the move count")]
    InvalidMoveCount,
}

impl Board {
    /// Convert the current board state into [Forsyth-Edwards
    /// Notation](https://en.wikipedia.org/wiki/Forsyth%E2%80%93Edwards_Notation).
    pub fn fen(&self) -> String {
        let mut result = String::new();

        push_placement(&self, &mut result);
        result.push(' ');

        result.push(if self.to_move.is_white() { 'w' } else { 'b' });
        result.push(' ');

        result.push_str(&self.castling.as_fen_str());
        result.push(' ');

        match self.en_passant {
            Some(square) => result.push_str(&square.to_string()),
            None => result.push('-'),
        };
        result.push(' ');

        result.push_str(&self.halfmove_clock.to_string());
        result.push(' ');

        result.push_str(&self.fullmoves.to_string());

        result
    }

    /// Parse the provided FEN ([Forsyth-Edwards
    /// Notation](https://en.wikipedia.org/wiki/Forsyth%E2%80%93Edwards_Notation))
    /// string into a [`Board`].
    pub fn try_parse_fen(fen: &str) -> Result<Board, FenParseError> {
        let fields: Vec<_> = fen.split(' ').collect();
        if fields.len() != 6 {
            return Err(FenParseError::IncorrectFieldCount {
                actual: fields.len(),
            });
        }

        let board = parse_placement(fields[0])?;
        let to_move = parse_to_move(fields[1])?;
        let castling = parse_castling(fields[2])?;
        let en_passant = parse_en_passant(fields[3])?;

        let halfmove_clock = fields[4]
            .parse()
            .map_err(|_| FenParseError::InvalidHalfMoveClock)?;
        let fullmoves = fields[5]
            .parse()
            .map_err(|_| FenParseError::InvalidMoveCount)?;
        if fullmoves <= 0 {
            return Err(FenParseError::InvalidMoveCount);
        }

        Ok(Board::new(
            board,
            to_move,
            castling,
            en_passant,
            halfmove_clock,
            fullmoves,
        ))
    }
}

fn push_placement(board: &Board, result: &mut String) {
    for rank in (0..8).rev() {
        let mut empty_squares = 0;

        for file in 0..8 {
            let square = Square::new_unchecked(rank, file);
            match board.board.piece_at(square) {
                None => empty_squares += 1,
                Some(piece) => {
                    if empty_squares > 0 {
                        result.push((empty_squares + b'0') as _);
                    }
                    result.push(piece.as_fen_char());
                    empty_squares = 0;
                }
            }
        }

        if empty_squares > 0 {
            result.push((empty_squares + b'0') as _);
        }

        if rank > 0 {
            result.push('/');
        }
    }
}

fn parse_placement(placement: &str) -> Result<[Option<Piece>; 64], FenParseError> {
    let ranks: Vec<_> = placement.split('/').collect();
    if ranks.len() != 8 {
        return Err(FenParseError::IncorrectRankCount {
            actual: ranks.len(),
        });
    }

    let mut board = [None; 64];
    for (rank_index, rank_pieces) in ranks.into_iter().enumerate() {
        let rank = 7 - rank_index as u8;
        let mut file = 0usize;

        for c in rank_pieces.chars() {
            if c.is_ascii_digit() {
                let digit = (c as u8) - b'0';
                file += digit as usize;
            } else {
                let piece =
                    Piece::try_from_fen_char(c).ok_or(FenParseError::UnknownPiece { piece: c })?;

                // If file >= 8, then we already know this FEN is invalid.
                // We'll report it later
                if file < 8 {
                    let index = Square::new_unchecked(rank, file as u8).index();
                    board[index] = Some(piece);
                }

                file += 1;
            }
        }

        if file != 8 {
            return Err(FenParseError::IncorrectFileCount { rank, actual: file });
        }
    }

    Ok(board)
}

fn parse_to_move(to_move: &str) -> Result<Color, FenParseError> {
    match to_move {
        "w" => Ok(Color::White),
        "b" => Ok(Color::Black),
        _ => Err(FenParseError::InvalidCurrentPlayer),
    }
}

fn parse_castling(castling: &str) -> Result<Castling, FenParseError> {
    if castling == "-" {
        Ok(Castling::empty())
    } else {
        let mut flags = Castling::empty();
        for c in castling.chars() {
            match c {
                'K' => flags |= Castling::WHITE_KINGSIDE,
                'Q' => flags |= Castling::WHITE_QUEENSIDE,
                'k' => flags |= Castling::BLACK_KINGSIDE,
                'q' => flags |= Castling::BLACK_QUEENSIDE,
                _ => return Err(FenParseError::InvalidCastling),
            }
        }

        Ok(flags)
    }
}

fn parse_en_passant(en_passant: &str) -> Result<Option<Square>, FenParseError> {
    if en_passant == "-" {
        Ok(None)
    } else {
        Ok(Some(
            en_passant
                .parse()
                .map_err(|_| FenParseError::InvalidEnPassant)?,
        ))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::board::STARTING_POSITION_FEN;

    const VALID_FENS: &[&str] = &[
        STARTING_POSITION_FEN,
        "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1",
        "rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 2",
        "rnbqkbnr/ppp1pppp/8/3P4/8/8/PPPP1PPP/RNBQKBNR b KQkq - 0 2",
        "rnb1kbnr/ppp1pppp/8/3q4/8/8/PPPP1PPP/RNBQKBNR w KQkq - 0 3",
        "rnb1kbnr/ppp1pppp/8/3q4/8/2N5/PPPP1PPP/R1BQKBNR b KQkq - 1 3",
        "rnb1kbnr/ppp1pppp/8/q7/8/2N5/PPPP1PPP/R1BQKBNR w KQkq - 2 4",
        "r1bq1rk1/pppp1ppp/2n5/1Bb5/3pP1nP/5N1R/PPP2PP1/RNBQK3 w Q - 2 8",
        "2kr1b1r/ppq2p1p/1np1ppb1/8/3P1P2/1BP3N1/PP1BQ1PP/R3K2R w KQ - 1 15",
        "2kr1b1r/ppq2p1p/1np1ppb1/8/3P1P2/1BP3N1/PP1BQ1PP/R4RK1 b - - 2 15",
        "8/4n2k/p4N2/1p1p4/1P1P1Bb1/2K5/1P6/8 w - - 2 45",
        "8/8/5K2/8/2k5/8/5Q2/8 w - - 12 49",
    ];

    const INVALID_FENS: &[&str] = &[
        "",
        " ",
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1 x",
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0",
        "rnbqkbnr/pppppppp/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        "rnbqkbnr/pppppppp/9/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        "rnbqkbnr/pppppppp/8/8/8/8/P1PPPP2P/RNBQKBNR w KQkq - 0 1",
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPPP/RNBQKBNR w KQkq - 0 1",
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR x KQkq - 0 1",
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR x KQkq - 0 1",
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR x KQkq - 0 1",
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w m - 0 1",
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w mKQkq - 0 1",
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 3.5 5",
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 5.1",
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 0",
    ];

    #[test]
    fn parse_valid_fens() {
        for fen in VALID_FENS {
            assert_eq!(&Board::try_parse_fen(fen).unwrap().fen(), fen);
        }
    }

    #[test]
    fn parse_invalid_fens() {
        for fen in INVALID_FENS {
            assert!(Board::try_parse_fen(fen).is_err());
        }
    }
}
