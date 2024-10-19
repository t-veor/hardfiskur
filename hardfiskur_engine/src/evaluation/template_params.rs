use hardfiskur_core::board::PieceType;

pub trait ColorParam: sealed::Sealed {
    const IS_WHITE: bool;
    const IS_BLACK: bool = !Self::IS_WHITE;

    const SIGN: i32 = if Self::IS_WHITE { 1 } else { -1 };
    const COEFF: i16 = Self::SIGN as i16;
}

pub struct White;
pub struct Black;

impl ColorParam for White {
    const IS_WHITE: bool = true;
}

impl ColorParam for Black {
    const IS_WHITE: bool = false;
}

pub trait PieceTypeParam: sealed::Sealed {
    const PIECE_TYPE: PieceType;
    const INDEX: usize = Self::PIECE_TYPE.index();
}

pub struct Pawn;
pub struct Knight;
pub struct Bishop;
pub struct Rook;
pub struct Queen;
pub struct King;

impl PieceTypeParam for Pawn {
    const PIECE_TYPE: PieceType = PieceType::Pawn;
}

impl PieceTypeParam for Knight {
    const PIECE_TYPE: PieceType = PieceType::Knight;
}

impl PieceTypeParam for Bishop {
    const PIECE_TYPE: PieceType = PieceType::Bishop;
}

impl PieceTypeParam for Rook {
    const PIECE_TYPE: PieceType = PieceType::Rook;
}

impl PieceTypeParam for Queen {
    const PIECE_TYPE: PieceType = PieceType::Queen;
}

impl PieceTypeParam for King {
    const PIECE_TYPE: PieceType = PieceType::King;
}

mod sealed {
    pub trait Sealed {}

    impl Sealed for super::White {}
    impl Sealed for super::Black {}
    impl Sealed for super::Pawn {}
    impl Sealed for super::Knight {}
    impl Sealed for super::Bishop {}
    impl Sealed for super::Rook {}
    impl Sealed for super::Queen {}
    impl Sealed for super::King {}
}
