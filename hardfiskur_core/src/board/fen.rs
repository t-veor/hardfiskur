use super::{Board, Castling, Color, Piece, Square};

pub fn board_to_fen(board: &Board) -> String {
    let mut result = String::new();

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

    result.push(' ');

    result.push(' ');
    result.push(if board.to_move.is_white() { 'w' } else { 'b' });
    result.push(' ');
    result.push_str(&board.castling.as_fen_str());
    result.push(' ');
    match board.en_passant {
        Some(square) => result.push_str(&square.to_string()),
        None => result.push('-'),
    };

    result.push(' ');
    result.push_str(&board.halfmove_clock.to_string());
    result.push(' ');
    result.push_str(&board.fullmoves.to_string());

    result
}

pub fn try_parse_fen(fen: &str) -> Option<Board> {
    let fields: Vec<_> = fen.split(' ').collect();
    if fields.len() != 6 {
        return None;
    }
    let placement = fields[0];
    let to_move = fields[1];
    let castling = fields[2];
    let en_passant = fields[3];
    let halfmove_clock = fields[4];
    let fullmoves = fields[5];

    let ranks: Vec<_> = placement.split('/').collect();
    if ranks.len() != 8 {
        return None;
    }

    let mut board = [None; 64];
    for (rank_index, rank_pieces) in ranks.into_iter().enumerate() {
        let rank = 7 - rank_index as u8;
        let mut file = 0;
        for char in rank_pieces.chars() {
            if char.is_ascii_digit() {
                let digit = (char as u8) - b'0';
                file += digit;
            } else {
                let piece = Piece::try_from_fen_char(char)?;
                let index = Square::new_unchecked(rank, file).index();
                board[index] = Some(piece);

                file += 1
            }

            if file >= 8 {
                break;
            }
        }
    }

    let to_move = match to_move {
        "w" => Color::White,
        "b" => Color::Black,
        _ => return None,
    };

    let castling = if castling == "-" {
        Castling::empty()
    } else {
        let mut flags = Castling::empty();
        for c in castling.chars() {
            match c {
                'K' => flags |= Castling::WHITE_KINGSIDE,
                'Q' => flags |= Castling::WHITE_QUEENSIDE,
                'k' => flags |= Castling::BLACK_KINGSIDE,
                'q' => flags |= Castling::BLACK_QUEENSIDE,
                _ => return None,
            }
        }
        flags
    };

    let en_passant = if en_passant == "-" {
        None
    } else {
        Some(en_passant.parse().ok()?)
    };

    let halfmove_clock = halfmove_clock.parse().ok()?;
    let fullmoves = fullmoves.parse().ok()?;

    Some(Board::new(
        board,
        to_move,
        castling,
        en_passant,
        halfmove_clock,
        fullmoves,
    ))
}
