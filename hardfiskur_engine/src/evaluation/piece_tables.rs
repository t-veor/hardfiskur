use hardfiskur_core::board::{PieceType, Square};

#[rustfmt::skip]
mod tables {
    // Janky macro to flip the order of the rows, because we'd like the rows to
    // match the chessboard from our perspective, but index 0 (top-left) is
    // actually a1 (bottom-left) in our square representation.
    // This macro matches groups of 8 elements and reverses the order of the
    // groups.
    macro_rules! flipped_8 {
        ($($remaining:expr),* $(,)?) => {
            flipped_8![$($remaining),* , ;]
        };
        (
            $a:expr, $b:expr, $c:expr, $d:expr, $e:expr, $f:expr, $g:expr, $h:expr,
            $($remaining:expr,)*
            ;
            $($rest:expr,)*
        ) => {
            flipped_8![
                $($remaining,)*
                ;
                $a, $b, $c, $d, $e, $f, $g, $h,
                $($rest,)*
            ]
        };
        (; $($rest:expr,)*) => {
            [$($rest,)*]
        };
    }

    pub const PAWN: [i32; 64] = flipped_8![
           0,   0,   0,   0,   0,   0,   0,   0,
         500, 500, 500, 500, 500, 500, 500, 500,
         100, 100, 200, 300, 300, 200, 100, 100,
          50,  50, 100, 250, 250, 100,  50,  50,
           0,   0,   0, 200, 200,   0,   0,   0,
          50, -50,-100,   0,   0,-100, -50,  50,
          50, 100, 100,-200,-200, 100, 100,  50,
           0,   0,   0,   0,   0,   0,   0,   0,
    ];

    pub const KNIGHT: [i32; 64] = flipped_8![
        -500,-400,-300,-300,-300,-300,-400,-500,
        -400,-200,   0,   0,   0,   0,-200,-400,
        -300,   0, 100, 150, 150, 100,   0,-300,
        -300,  50, 150, 200, 200, 150,  50,-300,
        -300,   0, 150, 200, 200, 150,   0,-300,
        -300,  50, 100, 150, 150, 100,  50,-300,
        -400,-200,   0,  50,  50,   0,-200,-400,
        -500,-400,-300,-300,-300,-300,-400,-500,
    ];

    pub const BISHOP: [i32; 64] = flipped_8![
        -200,-100,-100,-100,-100,-100,-100,-200,
        -100,   0,   0,   0,   0,   0,   0,-100,
        -100,   0,  50, 100, 100,  50,   0,-100,
        -100,  50,  50, 100, 100,  50,  50,-100,
        -100,   0, 100, 100, 100, 100,   0,-100,
        -100, 100, 100, 100, 100, 100, 100,-100,
        -100,  50,   0,   0,   0,   0,  50,-100,
        -200,-100,-100,-100,-100,-100,-100,-200,
    ];

    pub const ROOK: [i32; 64] = flipped_8![
           0,   0,   0,   0,   0,   0,   0,   0,
          50, 100, 100, 100, 100, 100, 100,  50,
         -50,   0,   0,   0,   0,   0,   0, -50,
         -50,   0,   0,   0,   0,   0,   0, -50,
         -50,   0,   0,   0,   0,   0,   0, -50,
         -50,   0,   0,   0,   0,   0,   0, -50,
         -50,   0,   0,   0,   0,   0,   0, -50,
           0,   0,   0,  50,  50,   0,   0,   0,
    ];

    pub const QUEEN: [i32; 64] = flipped_8![
        -200,-100,-100, -50, -50,-100,-100,-200,
        -100,   0,   0,   0,   0,   0,   0,-100,
        -100,   0,  50,  50,  50,  50,   0,-100,
         -50,   0,  50,  50,  50,  50,   0, -50,
           0,   0,  50,  50,  50,  50,   0, -50,
        -100,  50,  50,  50,  50,  50,   0,-100,
        -100,   0,  50,   0,   0,   0,   0,-100,
        -200,-100,-100, -50, -50,-100,-100,-200,
    ];

    pub const KING_MIDDLE_GAME: [i32; 64] = flipped_8![
        -300,-400,-400,-500,-500,-400,-400,-300,
        -300,-400,-400,-500,-500,-400,-400,-300,
        -300,-400,-400,-500,-500,-400,-400,-300,
        -300,-400,-400,-500,-500,-400,-400,-300,
        -200,-300,-300,-400,-400,-300,-300,-200,
        -100,-200,-200,-200,-200,-200,-200,-100,
         200, 200,   0,   0,   0,   0, 200, 200,
         200, 300, 100,   0,   0, 100, 300, 200,
    ];

    pub const KING_END_GAME: [i32; 64] = flipped_8![
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

pub const fn piece_square_table(piece_type: PieceType, square: Square) -> (i32, i32) {
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
