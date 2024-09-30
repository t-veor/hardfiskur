use hardfiskur_core::board::{PieceType, Square};

#[rustfmt::skip]
mod tables {
    pub const PAWN: [i32; 64] = [
           0,   0,   0,   0,   0,   0,   0,   0,
         500, 500, 500, 500, 500, 500, 500, 500,
         100, 100, 200, 300, 300, 200, 100, 100,
          50,  50, 100, 250, 250, 100,  50,  50,
           0,   0,   0, 200, 200,   0,   0,   0,
          50, -50,-100,   0,   0,-100, -50,  50,
          50, 100, 100,-200,-200, 100, 100,  50,
           0,   0,   0,   0,   0,   0,   0,   0,
    ];

    pub const KNIGHT: [i32; 64] = [
        -500,-400,-300,-300,-300,-300,-400,-500,
        -400,-200,   0,   0,   0,   0,-200,-400,
        -300,   0, 100, 150, 150, 100,   0,-300,
        -300,  50, 150, 200, 200, 150,  50,-300,
        -300,   0, 150, 200, 200, 150,   0,-300,
        -300,  50, 100, 150, 150, 100,  50,-300,
        -400,-200,   0,  50,  50,   0,-200,-400,
        -500,-400,-300,-300,-300,-300,-400,-500,
    ];

    pub const BISHOP: [i32; 64] = [
        -200,-100,-100,-100,-100,-100,-100,-200,
        -100,   0,   0,   0,   0,   0,   0,-100,
        -100,   0,  50, 100, 100,  50,   0,-100,
        -100,  50,  50, 100, 100,  50,  50,-100,
        -100,   0, 100, 100, 100, 100,   0,-100,
        -100, 100, 100, 100, 100, 100, 100,-100,
        -100,  50,   0,   0,   0,   0,  50,-100,
        -200,-100,-100,-100,-100,-100,-100,-200,
    ];

    pub const ROOK: [i32; 64] = [
           0,   0,   0,   0,   0,   0,   0,   0,
          50, 100, 100, 100, 100, 100, 100,  50,
         -50,   0,   0,   0,   0,   0,   0, -50,
         -50,   0,   0,   0,   0,   0,   0, -50,
         -50,   0,   0,   0,   0,   0,   0, -50,
         -50,   0,   0,   0,   0,   0,   0, -50,
         -50,   0,   0,   0,   0,   0,   0, -50,
           0,   0,   0,  50,  50,   0,   0,   0,
    ];

    pub const QUEEN: [i32; 64] = [
        -200,-100,-100, -50, -50,-100,-100,-200,
        -100,   0,   0,   0,   0,   0,   0,-100,
        -100,   0,  50,  50,  50,  50,   0,-100,
         -50,   0,  50,  50,  50,  50,   0, -50,
           0,   0,  50,  50,  50,  50,   0, -50,
        -100,  50,  50,  50,  50,  50,   0,-100,
        -100,   0,  50,   0,   0,   0,   0,-100,
        -200,-100,-100, -50, -50,-100,-100,-200,
    ];

    pub const KING_MIDDLE_GAME: [i32; 64] = [
        -300,-400,-400,-500,-500,-400,-400,-300,
        -300,-400,-400,-500,-500,-400,-400,-300,
        -300,-400,-400,-500,-500,-400,-400,-300,
        -300,-400,-400,-500,-500,-400,-400,-300,
        -200,-300,-300,-400,-400,-300,-300,-200,
        -100,-200,-200,-200,-200,-200,-200,-100,
         200, 200,   0,   0,   0,   0, 200, 200,
         200, 300, 100,   0,   0, 100, 300, 200,
    ];

    pub const KING_END_GAME: [i32; 64] = [
        -500,-400,-300,-200,-200,-300,-400,-500,
        -300,-200,-100,   0,   0,-100,-200,-300,
        -300,-100, 200, 300, 300, 200,-100,-300,
        -300,-100, 300, 400, 400, 300,-100,-300,
        -300,-100, 300, 400, 400, 300,-100,-300,
        -300,-100, 200, 300, 300, 200,-100,-300,
        -300,-300,   0,   0,   0,   0,-300,-300,
        -500,-300,-300,-300,-300,-300,-300,-500,
    ];
}

pub const fn material_score(piece_type: PieceType) -> i32 {
    match piece_type {
        PieceType::Pawn => 1000,
        PieceType::Knight => 3200,
        PieceType::Bishop => 3300,
        PieceType::Rook => 5000,
        PieceType::Queen => 9000,
        PieceType::King => 0,
    }
}

pub const FULL_ENDGAME_PHASE: i32 = 24;

pub const fn phase_modifier(piece_type: PieceType) -> i32 {
    match piece_type {
        PieceType::Knight => 1,
        PieceType::Bishop => 1,
        PieceType::Rook => 2,
        PieceType::Queen => 4,
        _ => 0,
    }
}

pub const fn piece_square_table(piece_type: PieceType, square: Square) -> (i32, i32) {
    let square = square.flip();

    let table = match piece_type {
        PieceType::Pawn => &tables::PAWN,
        PieceType::Knight => &tables::KNIGHT,
        PieceType::Bishop => &tables::BISHOP,
        PieceType::Rook => &tables::ROOK,
        PieceType::Queen => &tables::QUEEN,
        PieceType::King => {
            // Special handling.
            return (
                tables::KING_MIDDLE_GAME[square.index()],
                tables::KING_END_GAME[square.index()],
            );
        }
    };

    let val = table[square.index()];
    (val, val)
}
