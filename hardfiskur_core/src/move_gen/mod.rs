//! Move generation and lookup tables.

use arrayvec::ArrayVec;
use bitflags::bitflags;

use crate::board::{Bitboard, BoardRepr, Castling, Color, Move, Piece, PieceType, Square};

use lookups::Lookups;

pub use pseudo_legal::{black_pawn_attacks, white_pawn_attacks};

pub mod bitboard_utils;
pub mod lookups;
pub mod magic;
mod pseudo_legal;

/// Maximum number of moves that could occur in a legal position, used for
/// stack-allocating a vector to hold moves.
///
/// The actual number appears to be 218 in this position:
///
/// R6R/3Q4/1Q4Q1/4Q3/2Q4Q/Q4Q2/pp1Q4/kBNN1KB1 w - - 0 1
///
/// But 256 is a nice number and a good buffer in case there could be more.
pub const MAX_MOVES: usize = 256;

const POSSIBLE_PROMOTIONS: &[PieceType] = &[
    PieceType::Queen,
    PieceType::Knight,
    PieceType::Rook,
    PieceType::Bishop,
];

pub type MoveVec = ArrayVec<Move, MAX_MOVES>;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct MoveGenFlags: u8 {
        const GEN_CAPTURES = 0b01;
        const GEN_QUIET_MOVES = 0b10;
    }
}

impl Default for MoveGenFlags {
    fn default() -> Self {
        Self::all()
    }
}

#[derive(Debug, Clone, Default)]
pub struct MoveGenResult {
    pub checker_count: u32,
    pub en_passant_possible: bool,
}

/// Masks used by the pseudo-legal move generation methods that restrict the
/// kinds of moves produced.
///
/// These masks are typically all set to [`Bitboard::ALL`], but in cases where
/// there is a check or pin, they may be set so that the psuedo-legal move
/// generation methods handle checks and pins correctly, giving us legal move
/// generation for free without (much) special handling when the king is in
/// check.
///
/// The reason the capture and push masks are separate is due to en passant --
/// it may have sufficed to have a single "push" mask represent both squares
/// that can be moved onto and pieces that must be captured, except in the case
/// of en passant, the square that the capturing piece lands on is different to
/// the square that the captured piece is on. Two different masks are needed to
/// handle the two situations where a pawn which can be en passant captured is
/// giving check, and where capturing a pawn via en passant also blocks a check.
///
/// Inspiration for this style of move generation comes from
/// <https://peterellisjones.com/posts/generating-legal-chess-moves-efficiently/>.
#[derive(Debug, Clone)]
struct MoveGenMasks {
    /// Pieces are only capturable if they are in this mask. This is normally
    /// [`Bitboard::ALL`].
    ///
    /// If the king is in check (and there is only one checker), this mask will
    /// consist of only the square of the checker -- indicating that any
    /// capturing moves generated must capture the checker.
    capture: Bitboard,
    /// Squares can only be moved onto if they are in this mask. This is
    /// normally [`Bitboard::ALL`].
    ///
    /// If the king is in check by a sliding piece, this mask will consist of
    /// the squares in between the king and the checker -- indicating that any
    /// moves generated must result in the moved piece landing on a square that
    /// blocks the check.
    push: Bitboard,
    /// Pieces can only be moved if they are in this mask. This is normally
    /// [`Bitboard:ALL`].
    ///
    /// Pieces which are absolutely pinned will be filtered out of this mask and
    /// their move generation will be handled separately from the pseudo-legal
    /// move generation.
    movable: Bitboard,
}

impl Default for MoveGenMasks {
    fn default() -> Self {
        Self {
            capture: Bitboard::ALL,
            push: Bitboard::ALL,
            movable: Bitboard::ALL,
        }
    }
}

pub struct MoveGenerator<'board, 'moves> {
    lookups: &'static Lookups,
    board: &'board BoardRepr,
    to_move: Color,
    empty: Bitboard,
    occupied: Bitboard,
    en_passant: Option<Square>,
    castling: Castling,
    flags: MoveGenFlags,

    en_passant_possible: bool,
    out_moves: &'moves mut MoveVec,
}

impl<'board, 'moves> MoveGenerator<'board, 'moves> {
    pub fn new(
        board: &'board BoardRepr,
        to_move: Color,
        en_passant: Option<Square>,
        castling: Castling,
        flags: MoveGenFlags,
        out_moves: &'moves mut MoveVec,
    ) -> Self {
        Self {
            lookups: Lookups::get_instance(),
            board,
            to_move,
            empty: board.empty(),
            occupied: board.occupied(),
            en_passant,
            castling,
            flags,

            en_passant_possible: false,
            out_moves,
        }
    }

    pub fn legal_moves(&mut self) -> MoveGenResult {
        let mut push_mask = if self.flags.contains(MoveGenFlags::GEN_QUIET_MOVES) {
            Bitboard::ALL
        } else {
            Bitboard::EMPTY
        };

        let mut capture_mask = if self.flags.contains(MoveGenFlags::GEN_CAPTURES) {
            Bitboard::ALL
        } else {
            Bitboard::EMPTY
        };

        let king_bb = self.board[PieceType::King.with_color(self.to_move)];
        let king = king_bb
            .to_square()
            .expect("No kings encountered during move generation");

        let king_danger_squares = self.king_danger_squares(king_bb);

        // Can always generate legal moves for kings
        self.legal_king_moves(king, king_danger_squares, push_mask, capture_mask);

        // Is the king in check?
        let checkers = self.attackers_on_king(king);
        let checker_count = checkers.pop_count();

        if checker_count > 1 {
            // In double check, only legal moves are the king's, so we can bail
            return MoveGenResult {
                checker_count,
                en_passant_possible: false,
            };
        }

        if checker_count == 1 {
            // In check, must do one of:
            // 1. Move the king out of check (already handled by
            //    legal_king_moves())
            // 2. Capture the checking piece -- accomplish this by setting the
            //    capture mask to only the checking piece
            // 3. Block the checking piece (if it is a sliding piece) --
            //    accomplish this by setting the push_mask to only squares
            //    between the checker and the king

            capture_mask &= checkers;

            let checker_square = checkers.to_square().unwrap();
            let checker = self
                .board
                .piece_with_color_at(self.to_move.flip(), checker_square)
                .unwrap();

            push_mask &= if checker.is_slider() {
                self.lookups.get_in_between(king, checker_square)
            } else {
                // Blocking not possible, set push mask to empty
                Bitboard::EMPTY
            };
        }

        // Find and generate moves for absolutely pinned pieces
        let pinned_pieces =
            self.find_and_gen_moves_for_pinned_pieces(king, push_mask, capture_mask);

        let masks = MoveGenMasks {
            capture: capture_mask,
            push: push_mask,
            // Pieces that are not absolutely pinned can move normally, but
            // don't try to generate moves for absolutely pinned pieces (as
            // they were already handled)
            movable: !pinned_pieces,
        };

        // Remaining pieces can move normally as long as they abide by the
        // masks, which will make sure they deal with checks and pins correctly
        self.pseudo_legal_moves(&masks);

        if checker_count == 0 && self.flags.contains(MoveGenFlags::GEN_QUIET_MOVES) {
            // Castling may be possible
            self.castling_moves(king, king_danger_squares);
        }

        MoveGenResult {
            checker_count,
            en_passant_possible: self.en_passant_possible,
        }
    }

    fn attackers_on_king(&self, king_square: Square) -> Bitboard {
        let mut attackers = Bitboard::EMPTY;
        let b = Bitboard::from_square(king_square);
        let opponent = self.to_move.flip();

        let pawn_attack_pattern = if self.to_move.is_white() {
            b.step_north_east() | b.step_north_west()
        } else {
            b.step_south_east() | b.step_south_west()
        };
        attackers |= pawn_attack_pattern & self.board[PieceType::Pawn.with_color(opponent)];

        attackers |= self.lookups.get_knight_moves(king_square)
            & self.board[PieceType::Knight.with_color(opponent)];

        attackers |= self.lookups.get_bishop_attacks(self.occupied, king_square)
            & (self.board[PieceType::Bishop.with_color(opponent)]
                | self.board[PieceType::Queen.with_color(opponent)]);

        attackers |= self.lookups.get_rook_attacks(self.occupied, king_square)
            & (self.board[PieceType::Rook.with_color(opponent)]
                | self.board[PieceType::Queen.with_color(opponent)]);

        // No need to check for king attacks, it's not possible for kings to be
        // adjacent in legal positions

        attackers
    }

    fn king_danger_squares(&self, king_bb: Bitboard) -> Bitboard {
        let occupied_without_king = self.occupied.without(king_bb);
        attacked_squares(
            self.board,
            self.to_move,
            self.lookups,
            occupied_without_king,
        )
    }

    fn legal_king_moves(
        &mut self,
        king: Square,
        king_danger_squares: Bitboard,
        push_mask: Bitboard,
        capture_mask: Bitboard,
    ) {
        let piece = PieceType::King.with_color(self.to_move);

        let pushable_squares = self.empty & push_mask;
        let capturable_pieces = self.board[self.to_move.flip()] & capture_mask;

        let attack_pattern = self
            .lookups
            .get_king_moves(king)
            .without(king_danger_squares);

        let pushes = attack_pattern & pushable_squares;
        let captures = attack_pattern & capturable_pieces;

        for to in pushes.squares() {
            self.out_moves.push(Move::builder(king, to, piece).build());
        }

        for to in captures.squares() {
            self.out_moves.push(
                Move::builder(king, to, piece)
                    .captures(
                        self.board
                            .piece_with_color_at(self.to_move.flip(), to)
                            .unwrap(),
                    )
                    .build(),
            )
        }
    }

    fn find_and_gen_moves_for_pinned_pieces(
        &mut self,
        king: Square,
        push_mask: Bitboard,
        capture_mask: Bitboard,
    ) -> Bitboard {
        let mut pinned_pieces = Bitboard::EMPTY;

        let opponent = self.to_move.flip();
        let opponent_bishops = self.board[PieceType::Bishop.with_color(opponent)];
        let opponent_rooks = self.board[PieceType::Rook.with_color(opponent)];
        let opponent_queens = self.board[PieceType::Queen.with_color(opponent)];

        let own_pieces = self.board[self.to_move];

        let rook_pinners = xray_rook_attacks(self.occupied, own_pieces, self.lookups, king)
            & (opponent_rooks | opponent_queens);
        let bishop_pinners = xray_bishop_attacks(self.occupied, own_pieces, self.lookups, king)
            & (opponent_bishops | opponent_queens);

        for rook_pinner in rook_pinners.squares() {
            let in_between = self.lookups.get_in_between(rook_pinner, king);
            let pinned = in_between & own_pieces;

            pinned_pieces |= pinned;

            let pinned_square = pinned.to_square().unwrap();
            let pinned_piece = self
                .board
                .piece_with_color_at(self.to_move, pinned_square)
                .unwrap();

            // Generate special moves for pieces pinned orthogonally, by
            // creating special pin masks which only allow the pinner to be
            // captured and the pinned piece to move on all squares between the
            // king and the pinner
            let special_pin_masks = MoveGenMasks {
                capture: capture_mask & Bitboard::from_square(rook_pinner),
                push: push_mask & (in_between ^ pinned),
                movable: pinned,
            };

            match pinned_piece.piece_type() {
                // If the pin is orthogonal, a pinned pawn may be able to push
                // forwards
                PieceType::Pawn => self.pseudo_legal_pawn_pushes(&special_pin_masks),
                // If the pinned piece is a piece that can move orthogonally, it
                // can move to all spaces between the pinner and the king, and
                // also capture the pinner
                PieceType::Rook | PieceType::Queen => self.gen_moves_for_pinned_slider(
                    pinned_square,
                    pinned_piece,
                    &special_pin_masks,
                ),
                _ => (),
            }
        }

        for bishop_pinner in bishop_pinners.squares() {
            let in_between = self.lookups.get_in_between(bishop_pinner, king);
            let pinned = in_between & own_pieces;

            pinned_pieces |= pinned;

            let pinned_square = pinned.to_square().unwrap();
            let pinned_piece = self
                .board
                .piece_with_color_at(self.to_move, pinned_square)
                .unwrap();

            // Similar logic to orthogonal pins as above, but for diagonal pins
            // instead
            let special_pin_masks = MoveGenMasks {
                capture: capture_mask & Bitboard::from_square(bishop_pinner),
                push: push_mask & (in_between ^ pinned),
                movable: pinned,
            };

            match pinned_piece.piece_type() {
                PieceType::Pawn => {
                    // If the pin is diagonal, a pinned pawn may be able to
                    // capture the pinner...
                    self.pseudo_legal_pawn_captures(&special_pin_masks);
                    // ...or move towards the pinner in an en passant capture
                    self.pseudo_legal_en_passants(&special_pin_masks);
                }
                // If the pinned piece is a piece that can move diagonally, it
                // can move to all the spaces between the pinner and the king,
                // and also capture the pinner
                PieceType::Bishop | PieceType::Queen => self.gen_moves_for_pinned_slider(
                    pinned_square,
                    pinned_piece,
                    &special_pin_masks,
                ),
                _ => (),
            }
        }

        pinned_pieces
    }

    fn gen_moves_for_pinned_slider(&mut self, from: Square, piece: Piece, masks: &MoveGenMasks) {
        for push in masks.push.squares() {
            self.out_moves
                .push(Move::builder(from, push, piece).build());
        }

        for capture in masks.capture.squares() {
            self.out_moves.push(
                Move::builder(from, capture, piece)
                    .captures(
                        self.board
                            .piece_with_color_at(self.to_move.flip(), capture)
                            .unwrap(),
                    )
                    .build(),
            );
        }
    }

    fn castling_moves(&mut self, king_square: Square, king_danger_squares: Bitboard) {
        let castle_mask = match self.to_move {
            Color::White => Castling::WHITE,
            Color::Black => Castling::BLACK,
        };

        let can_castle_kingside = (self.castling & castle_mask).intersects(Castling::KINGSIDE);
        let can_castle_queenside = (self.castling & castle_mask).intersects(Castling::QUEENSIDE);

        let mut try_castle = |pass_through_file: u8, to_file: u8, rook_start_file: u8| {
            let from = king_square;
            let rook_square = Square::new_unchecked(from.rank(), rook_start_file);
            let pass_through_square = Square::new_unchecked(from.rank(), pass_through_file);
            let to = Square::new_unchecked(from.rank(), to_file);

            let rook = PieceType::Rook.with_color(self.to_move);
            // Check there actually is a rook to castle with
            if !self.board[rook].get(rook_square) {
                return;
            }

            let in_between = self.lookups.get_in_between(king_square, rook_square);
            // Can't castle if there are pieces between the king and rook
            if (in_between & self.occupied).has_piece() {
                return;
            }

            // Can't castle if the king moves through or ends in check
            let risk_squares =
                Bitboard::from_square(pass_through_square) | Bitboard::from_square(to);
            if (risk_squares & king_danger_squares).has_piece() {
                return;
            }

            // Castling is possible
            self.out_moves.push(
                Move::builder(from, to, PieceType::King.with_color(self.to_move))
                    .is_castle()
                    .build(),
            );
        };

        if can_castle_kingside {
            try_castle(5, 6, 7);
        }

        if can_castle_queenside {
            try_castle(3, 2, 0);
        }
    }
}

pub fn attacked_squares(
    board: &BoardRepr,
    to_move: Color,
    lookups: &Lookups,
    occupied: Bitboard,
) -> Bitboard {
    let mut attacked_squares = Bitboard::EMPTY;
    let opponent = to_move.flip();

    attacked_squares |= if to_move.is_white() {
        black_pawn_attacks(board[Piece::BLACK_PAWN])
    } else {
        white_pawn_attacks(board[Piece::WHITE_PAWN])
    };

    fn get_all_attacks<F>(pieces: Bitboard, get_attack_pattern: F) -> Bitboard
    where
        F: Fn(Square) -> Bitboard,
    {
        pieces
            .squares()
            .map(get_attack_pattern)
            .fold(Bitboard::EMPTY, Bitboard::or)
    }

    attacked_squares |= get_all_attacks(board[PieceType::Knight.with_color(opponent)], |square| {
        lookups.get_knight_moves(square)
    });

    attacked_squares |= get_all_attacks(
        board[PieceType::Bishop.with_color(opponent)]
            | board[PieceType::Queen.with_color(opponent)],
        |square| lookups.get_bishop_attacks(occupied, square),
    );

    attacked_squares |= get_all_attacks(
        board[PieceType::Rook.with_color(opponent)] | board[PieceType::Queen.with_color(opponent)],
        |square| lookups.get_rook_attacks(occupied, square),
    );

    attacked_squares |= get_all_attacks(board[PieceType::King.with_color(opponent)], |square| {
        lookups.get_king_moves(square)
    });

    attacked_squares
}

pub fn attackers_on(
    board: &BoardRepr,
    occupied: Bitboard,
    square: Square,
    lookups: &Lookups,
) -> Bitboard {
    let mut attackers = Bitboard::EMPTY;
    let b = Bitboard::from_square(square);

    attackers |= (b.step_south_east() | b.step_south_west()) & board[Piece::pawn(Color::White)];
    attackers |= (b.step_north_east() | b.step_north_west()) & board[Piece::pawn(Color::Black)];

    attackers |= lookups.get_knight_moves(square)
        & (board[Piece::knight(Color::White)] | board[Piece::knight(Color::Black)]);

    attackers |= lookups.get_bishop_attacks(occupied, square)
        & (board[Piece::bishop(Color::White)]
            | board[Piece::bishop(Color::Black)]
            | board[Piece::queen(Color::White)]
            | board[Piece::queen(Color::Black)]);

    attackers |= lookups.get_rook_attacks(occupied, square)
        & (board[Piece::rook(Color::White)]
            | board[Piece::rook(Color::Black)]
            | board[Piece::queen(Color::White)]
            | board[Piece::queen(Color::Black)]);

    attackers |= lookups.get_king_moves(square)
        & (board[Piece::king(Color::White)] | board[Piece::king(Color::Black)]);

    attackers
}

fn xray_bishop_attacks(
    occupied: Bitboard,
    xrayable_pieces: Bitboard,
    lookups: &Lookups,
    square: Square,
) -> Bitboard {
    let attacks = lookups.get_bishop_attacks(occupied, square);
    let pieces_to_remove = xrayable_pieces & attacks;
    attacks ^ lookups.get_bishop_attacks(occupied ^ pieces_to_remove, square)
}

fn xray_rook_attacks(
    occupied: Bitboard,
    xrayable_pieces: Bitboard,
    lookups: &Lookups,
    square: Square,
) -> Bitboard {
    let attacks = lookups.get_rook_attacks(occupied, square);
    let pieces_to_remove = xrayable_pieces & attacks;
    attacks ^ lookups.get_rook_attacks(occupied ^ pieces_to_remove, square)
}

#[cfg(test)]
mod test {
    use crate::test_utils::assert_in_any_order;

    use super::*;

    fn legal_moves_default_settings(board: &BoardRepr, color: Color) -> MoveVec {
        let mut moves = MoveVec::new();

        let mut move_gen = MoveGenerator::new(
            board,
            color,
            None,
            Default::default(),
            Default::default(),
            &mut moves,
        );

        move_gen.legal_moves();

        moves
    }

    #[test]
    fn normal_king_moves() {
        let board = "
            .....k..
            ........
            ........
            ........
            ........
            ........
            ..K.....
            ........
        "
        .parse()
        .unwrap();

        let white_moves = legal_moves_default_settings(&board, Color::White);
        let black_moves = legal_moves_default_settings(&board, Color::Black);

        assert_in_any_order(
            white_moves,
            vec![
                Move::builder(Square::C2, Square::B2, Piece::WHITE_KING).build(),
                Move::builder(Square::C2, Square::B3, Piece::WHITE_KING).build(),
                Move::builder(Square::C2, Square::C3, Piece::WHITE_KING).build(),
                Move::builder(Square::C2, Square::D3, Piece::WHITE_KING).build(),
                Move::builder(Square::C2, Square::D2, Piece::WHITE_KING).build(),
                Move::builder(Square::C2, Square::D1, Piece::WHITE_KING).build(),
                Move::builder(Square::C2, Square::C1, Piece::WHITE_KING).build(),
                Move::builder(Square::C2, Square::B1, Piece::WHITE_KING).build(),
            ],
        );

        assert_in_any_order(
            black_moves,
            vec![
                Move::builder(Square::F8, Square::E8, Piece::BLACK_KING).build(),
                Move::builder(Square::F8, Square::G8, Piece::BLACK_KING).build(),
                Move::builder(Square::F8, Square::G7, Piece::BLACK_KING).build(),
                Move::builder(Square::F8, Square::F7, Piece::BLACK_KING).build(),
                Move::builder(Square::F8, Square::E7, Piece::BLACK_KING).build(),
            ],
        );
    }

    #[test]
    fn king_moves_in_check() {
        let board = "
            .....k..
            ........
            ..r.....
            ........
            ....n...
            .p......
            ..K.....
            ........
        "
        .parse()
        .unwrap();

        let moves = legal_moves_default_settings(&board, Color::White);

        assert_in_any_order(
            moves,
            vec![
                Move::builder(Square::C2, Square::B1, Piece::WHITE_KING).build(),
                Move::builder(Square::C2, Square::B2, Piece::WHITE_KING).build(),
                Move::builder(Square::C2, Square::B3, Piece::WHITE_KING)
                    .captures(Piece::BLACK_PAWN)
                    .build(),
                Move::builder(Square::C2, Square::D3, Piece::WHITE_KING).build(),
                Move::builder(Square::C2, Square::D1, Piece::WHITE_KING).build(),
            ],
        )
    }

    #[test]
    fn double_check_generates_only_king_moves() {
        let board = "
            .....k..
            ........
            ..r.....
            ......R.
            ....n...
            .p......
            ..K.....
            ........
        "
        .parse()
        .unwrap();

        let moves = legal_moves_default_settings(&board, Color::White);

        assert_in_any_order(
            moves,
            vec![
                Move::builder(Square::C2, Square::B1, Piece::WHITE_KING).build(),
                Move::builder(Square::C2, Square::B2, Piece::WHITE_KING).build(),
                Move::builder(Square::C2, Square::B3, Piece::WHITE_KING)
                    .captures(Piece::BLACK_PAWN)
                    .build(),
                Move::builder(Square::C2, Square::D3, Piece::WHITE_KING).build(),
                Move::builder(Square::C2, Square::D1, Piece::WHITE_KING).build(),
            ],
        )
    }

    #[test]
    fn single_check_generates_blocking_moves() {
        let board = "
            r....k..
            ........
            ..N.....
            ....R...
            .P......
            KP..B...
            .P......
            ........
        "
        .parse()
        .unwrap();

        let moves = legal_moves_default_settings(&board, Color::White);

        assert_in_any_order(
            moves,
            vec![
                Move::builder(Square::C6, Square::A7, Piece::WHITE_KNIGHT).build(),
                Move::builder(Square::C6, Square::A5, Piece::WHITE_KNIGHT).build(),
                Move::builder(Square::E5, Square::A5, Piece::WHITE_ROOK).build(),
                Move::builder(Square::E3, Square::A7, Piece::WHITE_BISHOP).build(),
            ],
        )
    }

    #[test]
    fn single_check_generates_capturing_moves() {
        let board = "
            r....k..
            ........
            ..B.....
            .P......
            KP......
            .P......
            ........
            ........
        "
        .parse()
        .unwrap();

        let moves = legal_moves_default_settings(&board, Color::White);

        assert_in_any_order(
            moves,
            vec![Move::builder(Square::C6, Square::A8, Piece::WHITE_BISHOP)
                .captures(Piece::BLACK_ROOK)
                .build()],
        );
    }

    #[test]
    fn pinned_knights_cant_move() {
        let board = "
            ..k.....
            ........
            ........
            ........
            ....b...
            ........
            .....PNP
            .r.N...K
        "
        .parse()
        .unwrap();

        let moves = legal_moves_default_settings(&board, Color::White);

        assert_in_any_order(
            moves
                .into_iter()
                .filter(|m| m.piece() == Piece::WHITE_KNIGHT),
            vec![],
        )
    }

    #[test]
    fn vertically_pinned_pawns_can_push() {
        let board = "
            ....k...
            ....q...
            ........
            ........
            ........
            ........
            ....P...
            ....K...
        "
        .parse()
        .unwrap();

        let moves = legal_moves_default_settings(&board, Color::White);

        assert_in_any_order(
            moves.into_iter().filter(|m| m.piece() == Piece::WHITE_PAWN),
            vec![
                Move::builder(Square::E2, Square::E3, Piece::WHITE_PAWN).build(),
                Move::builder(Square::E2, Square::E4, Piece::WHITE_PAWN)
                    .is_double_pawn_push()
                    .build(),
            ],
        )
    }

    #[test]
    fn orthogonally_pinned_sliders() {
        let board = "
            ....k...
            ....q...
            ........
            ....R...
            ........
            r.Q.K.Br
            ........
            ........
        "
        .parse()
        .unwrap();

        let moves = legal_moves_default_settings(&board, Color::White);

        assert_in_any_order(
            moves.into_iter().filter(|m| {
                matches!(
                    m.piece(),
                    Piece::WHITE_BISHOP | Piece::WHITE_ROOK | Piece::WHITE_QUEEN
                )
            }),
            vec![
                Move::builder(Square::E5, Square::E4, Piece::WHITE_ROOK).build(),
                Move::builder(Square::E5, Square::E6, Piece::WHITE_ROOK).build(),
                Move::builder(Square::E5, Square::E7, Piece::WHITE_ROOK)
                    .captures(Piece::BLACK_QUEEN)
                    .build(),
                //
                Move::builder(Square::C3, Square::B3, Piece::WHITE_QUEEN).build(),
                Move::builder(Square::C3, Square::D3, Piece::WHITE_QUEEN).build(),
                Move::builder(Square::C3, Square::A3, Piece::WHITE_QUEEN)
                    .captures(Piece::BLACK_ROOK)
                    .build(),
            ],
        )
    }

    #[test]
    fn diagonally_pinned_pawns_can_capture() {
        let board = "
            ...k....
            ........
            b.......
            ...p.q..
            ..P.P...
            ...K....
            ........
            ........
        "
        .parse()
        .unwrap();

        let moves = legal_moves_default_settings(&board, Color::White);

        assert_in_any_order(
            moves.into_iter().filter(|m| m.piece() == Piece::WHITE_PAWN),
            vec![Move::builder(Square::E4, Square::F5, Piece::WHITE_PAWN)
                .captures(Piece::BLACK_QUEEN)
                .build()],
        );
    }

    #[test]
    fn diagonally_pinned_pawns_can_capture_en_passant_towards_pinner() {
        let board = "
            ...k...b
            ........
            ........
            ....Pp..
            ...K....
            ........
            ........
            ........
        "
        .parse()
        .unwrap();

        let mut moves = MoveVec::new();
        let mut move_gen = MoveGenerator::new(
            &board,
            Color::White,
            Some(Square::F6),
            Default::default(),
            Default::default(),
            &mut moves,
        );

        move_gen.legal_moves();

        assert_in_any_order(
            moves.into_iter().filter(|m| m.piece() == Piece::WHITE_PAWN),
            vec![Move::builder(Square::E5, Square::F6, Piece::WHITE_PAWN)
                .captures(Piece::BLACK_PAWN)
                .is_en_passant()
                .build()],
        );
    }

    #[test]
    fn diagonally_pinned_pawns_cannot_capture_en_passant() {
        let board = "
            ...k...b
            ........
            ........
            ...pP...
            ...K....
            ........
            ........
            ........
        "
        .parse()
        .unwrap();

        let mut moves = MoveVec::new();
        let mut move_gen = MoveGenerator::new(
            &board,
            Color::White,
            Some(Square::D6),
            Default::default(),
            Default::default(),
            &mut moves,
        );

        move_gen.legal_moves();

        assert_in_any_order(
            moves.into_iter().filter(|m| m.piece() == Piece::WHITE_PAWN),
            vec![],
        );
    }

    #[test]
    fn diagonally_pinned_sliders() {
        let board = "
            q...k...
            ........
            ..B...b.
            .....R..
            ....K...
            .....Q..
            ........
            .......b
        "
        .parse()
        .unwrap();

        let moves = legal_moves_default_settings(&board, Color::White);

        assert_in_any_order(
            moves.into_iter().filter(|m| {
                matches!(
                    m.piece(),
                    Piece::WHITE_BISHOP | Piece::WHITE_ROOK | Piece::WHITE_QUEEN
                )
            }),
            vec![
                Move::builder(Square::C6, Square::D5, Piece::WHITE_BISHOP).build(),
                Move::builder(Square::C6, Square::B7, Piece::WHITE_BISHOP).build(),
                Move::builder(Square::C6, Square::A8, Piece::WHITE_BISHOP)
                    .captures(Piece::BLACK_QUEEN)
                    .build(),
                //
                Move::builder(Square::F3, Square::G2, Piece::WHITE_QUEEN).build(),
                Move::builder(Square::F3, Square::H1, Piece::WHITE_QUEEN)
                    .captures(Piece::BLACK_BISHOP)
                    .build(),
            ],
        )
    }

    #[test]
    fn normal_castling_moves() {
        let board = "
            r...k..r
            ........
            ........
            ........
            ........
            ........
            ........
            R...K..R
        "
        .parse()
        .unwrap();

        let white_moves = legal_moves_default_settings(&board, Color::White);
        let black_moves = legal_moves_default_settings(&board, Color::Black);

        assert_in_any_order(
            white_moves.into_iter().filter(|m| m.is_castle()),
            vec![
                Move::builder(Square::E1, Square::G1, Piece::WHITE_KING)
                    .is_castle()
                    .build(),
                Move::builder(Square::E1, Square::C1, Piece::WHITE_KING)
                    .is_castle()
                    .build(),
            ],
        );

        assert_in_any_order(
            black_moves.into_iter().filter(|m| m.is_castle()),
            vec![
                Move::builder(Square::E8, Square::G8, Piece::BLACK_KING)
                    .is_castle()
                    .build(),
                Move::builder(Square::E8, Square::C8, Piece::BLACK_KING)
                    .is_castle()
                    .build(),
            ],
        );
    }

    #[test]
    fn cannot_castle_if_rook_missing() {
        let board = "
            ....k..n
            ........
            ........
            ........
            ........
            ........
            ........
            ....K..n
        "
        .parse()
        .unwrap();

        let white_moves = legal_moves_default_settings(&board, Color::White);
        let black_moves = legal_moves_default_settings(&board, Color::Black);

        assert_in_any_order(white_moves.into_iter().filter(|m| m.is_castle()), vec![]);

        assert_in_any_order(black_moves.into_iter().filter(|m| m.is_castle()), vec![]);
    }

    #[test]
    fn cannot_castle_if_rights_missing() {
        let board = "
            r...k..r
            ........
            ........
            ........
            ........
            ........
            ........
            R...K..R
        "
        .parse()
        .unwrap();

        let castling = Castling::WHITE_KINGSIDE | Castling::BLACK_QUEENSIDE;

        let white_moves = {
            let mut moves = MoveVec::new();

            let mut move_gen = MoveGenerator::new(
                &board,
                Color::White,
                None,
                castling,
                Default::default(),
                &mut moves,
            );

            move_gen.legal_moves();

            moves
        };

        let black_moves = {
            let mut moves = MoveVec::new();

            let mut move_gen = MoveGenerator::new(
                &board,
                Color::Black,
                None,
                castling,
                Default::default(),
                &mut moves,
            );

            move_gen.legal_moves();

            moves
        };

        assert_in_any_order(
            white_moves.into_iter().filter(|m| m.is_castle()),
            vec![Move::builder(Square::E1, Square::G1, Piece::WHITE_KING)
                .is_castle()
                .build()],
        );

        assert_in_any_order(
            black_moves.into_iter().filter(|m| m.is_castle()),
            vec![Move::builder(Square::E8, Square::C8, Piece::BLACK_KING)
                .is_castle()
                .build()],
        );
    }

    #[test]
    fn cannot_castle_if_pieces_between_king_and_rook() {
        let board = "
            rR..k.qr
            ........
            ........
            ........
            ........
            ........
            ........
            Rb..KQ.R
        "
        .parse()
        .unwrap();

        let white_moves = legal_moves_default_settings(&board, Color::White);
        let black_moves = legal_moves_default_settings(&board, Color::Black);

        assert_in_any_order(white_moves.into_iter().filter(|m| m.is_castle()), vec![]);

        assert_in_any_order(black_moves.into_iter().filter(|m| m.is_castle()), vec![]);
    }

    #[test]
    fn cannot_castle_in_check() {
        let board = "
            ....k...
            ........
            ....r...
            ........
            ........
            ........
            ........
            R...K..R
        "
        .parse()
        .unwrap();

        let moves = legal_moves_default_settings(&board, Color::White);

        assert_in_any_order(moves.into_iter().filter(|m| m.is_castle()), vec![]);
    }

    #[test]
    fn cannot_castle_through_check() {
        let board = "
            ....k...
            ........
            ........
            ........
            ........
            ........
            ....p...
            R...K..R
        "
        .parse()
        .unwrap();

        let moves = legal_moves_default_settings(&board, Color::White);

        assert_in_any_order(moves.into_iter().filter(|m| m.is_castle()), vec![]);
    }

    #[test]
    fn cannot_castle_into_check() {
        let board = "
            ....k...
            ........
            ........
            ........
            ........
            .n......
            .......b
            R...K..R
        "
        .parse()
        .unwrap();

        let moves = legal_moves_default_settings(&board, Color::White);

        assert_in_any_order(moves.into_iter().filter(|m| m.is_castle()), vec![]);
    }
}
