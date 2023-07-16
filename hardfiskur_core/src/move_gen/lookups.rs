//! Lookup tables and look up table generation functions.

use std::sync::OnceLock;

use num_traits::FromPrimitive;

use crate::board::{Bitboard, Square};

use super::{
    bitboard_utils::{king_moves, knight_attacks, unblocked_ray_attacks, Direction},
    magic::MagicTables,
};

/// Struct holding various lookup tables, useful for move generation.
///
/// This struct provides methods for obtaining the attack patterns of knights,
/// kings, bishops, and rooks given a starting square and blocking pieces, as
/// well as obtaining the squares in between two other squares.
///
/// These lookup tables are initialised once at the beginning of the program and
/// then should not change. The method [`Lookups::get_instance`] is provided to
/// initialise the tables on the first call, and then returns a cached `&'static
/// Lookups` which can be used wherever desired in the rest of the program.
pub struct Lookups {
    knight_moves: [Bitboard; 64],
    king_moves: [Bitboard; 64],
    in_between: [[Bitboard; 64]; 64],

    magic: &'static MagicTables,
}

static LOOKUPS: OnceLock<Lookups> = OnceLock::new();

impl Lookups {
    fn new() -> Self {
        let knight_moves = gen_knight_moves();
        let king_moves = gen_king_moves();
        let ray_attacks = gen_ray_attacks();
        let in_between = gen_in_between(&ray_attacks);

        let magic = MagicTables::get(&ray_attacks);

        Self {
            knight_moves,
            king_moves,
            in_between,

            magic,
        }
    }

    /// Get a static reference to the global [`Lookups`] instance.
    ///
    /// On the first call, this will populate and cache the lookup tables.
    /// Subsequent calls will simply return a reference to the cached lookup
    /// tables.
    pub fn get_instance() -> &'static Self {
        LOOKUPS.get_or_init(Self::new)
    }

    /// Gets all knight moves originating from the given square.
    pub fn get_knight_moves(&self, square: Square) -> Bitboard {
        self.knight_moves[square.index()]
    }

    /// Gets all king moves originating from the given square.
    pub fn get_king_moves(&self, square: Square) -> Bitboard {
        self.king_moves[square.index()]
    }

    /// Gets all rook moves originating from the given square.
    ///
    /// The given occupied bitboard will be used to block attacks. The first
    /// square encountered in each direction which is set in the occupied
    /// bitboard will be included in the result, but further squares will not.
    ///
    /// # Examples
    ///
    /// ```
    /// # use hardfiskur_core::{board::Square, move_gen::lookups::Lookups};
    /// let lookups = Lookups::get_instance();
    /// let occupied = "
    ///         . . . . . . . .
    ///         . . . . # . . .
    ///         . . . . . . . .
    ///         . . . . # . . .
    ///         . . # . . . . .
    ///         . . . . . . . .
    ///         . . . . . . . .
    ///         . . . . . . . .
    /// ".parse().unwrap();
    /// assert_eq!(
    ///     lookups.get_rook_attacks(occupied, Square::E4),
    ///     "
    ///         . . . . . . . .
    ///         . . . . . . . .
    ///         . . . . . . . .
    ///         . . . . # . . .
    ///         . . # # . # # #
    ///         . . . . # . . .
    ///         . . . . # . . .
    ///         . . . . # . . .
    ///     ".parse().unwrap(),
    /// );
    /// ```
    pub fn get_rook_attacks(&self, occupied: Bitboard, square: Square) -> Bitboard {
        self.magic.rook_attacks(occupied, square)
    }

    /// Gets all bishop moves originating from the given square.
    ///
    /// The given occupied bitboard will be used to block attacks. The first
    /// square encountered in each direction which is set in the occupied
    /// bitboard will be included in the result, but further squares will not.
    ///
    /// # Examples
    ///
    /// ```
    /// # use hardfiskur_core::{board::Square, move_gen::lookups::Lookups};
    /// let lookups = Lookups::get_instance();
    /// let occupied = "
    ///         . . . . . . . .
    ///         . # . . . . . .
    ///         . . . . . . . .
    ///         . . . # . . . .
    ///         . . . . . . . .
    ///         . . . . . . . .
    ///         . . # . . . . .
    ///         . . . . . . . .
    /// ".parse().unwrap();
    /// assert_eq!(
    ///     lookups.get_bishop_attacks(occupied, Square::E4),
    ///     "
    ///         . . . . . . . .
    ///         . . . . . . . #
    ///         . . . . . . # .
    ///         . . . # . # . .
    ///         . . . . . . . .
    ///         . . . # . # . .
    ///         . . # . . . # .
    ///         . . . . . . . #
    ///     ".parse().unwrap(),
    /// );
    /// ```
    pub fn get_bishop_attacks(&self, occupied: Bitboard, square: Square) -> Bitboard {
        self.magic.bishop_attacks(occupied, square)
    }

    /// Gets all queen moves originating from the given square.
    ///
    /// This is simply the union of the result of [`Self::get_rook_attacks`] and
    /// [`Self::get_bishop_attacks`].
    pub fn get_queen_attacks(&self, occupied: Bitboard, square: Square) -> Bitboard {
        self.get_rook_attacks(occupied, square) | self.get_bishop_attacks(occupied, square)
    }

    /// Gets the squares in between the two squares provided.
    ///
    /// If the two squares provided are on the same rank/file/diagonal, the
    /// bitboard returned will contain all squares between the `from` and `to`
    /// squares, excluding the `from` and `to` squares themselves. Otherwise, an
    /// empty bitboard is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// # use hardfiskur_core::{board::Square, move_gen::lookups::Lookups};
    /// let lookups = Lookups::get_instance();
    /// assert_eq!(
    ///     lookups.get_in_between(Square::C2, Square::H7),
    ///     "
    ///         . . . . . . . .
    ///         . . . . . . . .
    ///         . . . . . . # .
    ///         . . . . . # . .
    ///         . . . . # . . .
    ///         . . . # . . . .
    ///         . . . . . . . .
    ///         . . . . . . . .
    ///     ".parse().unwrap()
    /// );
    /// ```
    pub fn get_in_between(&self, from: Square, to: Square) -> Bitboard {
        self.in_between[from.index()][to.index()]
    }

    /// Returns the internal [`MagicTables`] instance for debugging purposes.
    ///
    /// This should not be used by the program normally but may be helpful in
    /// debugging or producing visualisations.
    pub fn debug_magic_tables(&self) -> &'static MagicTables {
        self.magic
    }
}

/// Generates a knight move lookup table.
///
/// The resulting table can be indexed by square index to retrieve the attack
/// pattern of a knight on that square.
pub fn gen_knight_moves() -> [Bitboard; 64] {
    let mut moves = [Bitboard::default(); 64];
    for (i, moves_from_square) in moves.iter_mut().enumerate() {
        *moves_from_square = knight_attacks(Bitboard::from_index(i));
    }
    moves
}

/// Generates a king move lookup table.
///
/// The resulting table can be indexed by square index to retrieve the moves of
/// a king on that square.
pub fn gen_king_moves() -> [Bitboard; 64] {
    let mut moves = [Bitboard::default(); 64];
    for (i, moves_from_square) in moves.iter_mut().enumerate() {
        *moves_from_square = king_moves(Bitboard::from_index(i));
    }
    moves
}

/// Generates a ray attack table.
///
/// The resulting table can be indexed by square, then by [`Direction`], to
/// retrieve the ray in that direction starting from the square.
pub fn gen_ray_attacks() -> [[Bitboard; 8]; 64] {
    let mut attacks = [[Bitboard::default(); 8]; 64];

    for (i, attacks_from_square) in attacks.iter_mut().enumerate() {
        let base = Bitboard::from_index(i);

        for (dir, attacks_in_dir) in attacks_from_square.iter_mut().enumerate() {
            let dir_enum = Direction::from_usize(dir).unwrap();
            *attacks_in_dir = unblocked_ray_attacks(base, dir_enum)
        }
    }

    attacks
}

/// Generates an in-between squares table.
///
/// The resulting table can be indexed by the starting and ending squares, to
/// retrieve all the squares between the starting and ending squares. If the
/// starting and ending squares are not on the same rank/file/diagonal, then an
/// empty bitboard is returned.
///
/// `ray_attacks` should be a valid ray attack table which is generated by
/// [`gen_ray_attacks`].
pub fn gen_in_between(ray_attacks: &[[Bitboard; 8]; 64]) -> [[Bitboard; 64]; 64] {
    let mut table = [[Bitboard::default(); 64]; 64];

    for from in 0..64 {
        for dir in 0..4 {
            let ray = ray_attacks[from][dir];
            for to in ray.bits() {
                let to = to as usize;
                let ray_between = ray ^ ray_attacks[to][dir] ^ Bitboard::from_index(to);
                table[from][to] = ray_between;
                table[to][from] = ray_between;
            }
        }
    }

    table
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn lookups_get_knight_moves() {
        let lookups = Lookups::get_instance();

        assert_eq!(
            lookups.get_knight_moves(Square::D4),
            "
                . . . . . . . .
                . . . . . . . .
                . . # . # . . .
                . # . . . # . .
                . . . . . . . .
                . # . . . # . .
                . . # . # . . .
                . . . . . . . .
            "
            .parse()
            .unwrap()
        );
    }

    #[test]
    fn lookups_get_king_moves() {
        let lookups = Lookups::get_instance();

        assert_eq!(
            lookups.get_king_moves(Square::D4),
            "
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . # # # . . .
                . . # . # . . .
                . . # # # . . .
                . . . . . . . .
                . . . . . . . .
            "
            .parse()
            .unwrap()
        );
    }

    #[test]
    fn lookups_get_rook_attacks() {
        let lookups = Lookups::get_instance();
        let occupied = "
                . . . . . . . .
                . . . . # . . .
                . . . . . . . .
                . . . . # . . .
                . . # . # . . .
                . . . . . . . .
                . . . # . . . .
                . . . . # . . .
        "
        .parse()
        .unwrap();
        assert_eq!(
            lookups.get_rook_attacks(occupied, Square::E4),
            "
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . # . . .
                . . # # . # # #
                . . . . # . . .
                . . . . # . . .
                . . . . # . . .
            "
            .parse()
            .unwrap(),
        );
    }

    #[test]
    fn lookups_get_bishop_attacks() {
        let lookups = Lookups::get_instance();
        let occupied = "
                . . . . . . . .
                . # . . . . . .
                . . . . . . . .
                . . . # . . . .
                . . . # # . . .
                . . . . . . . .
                . . # . . . . .
                . . . . . . . .
        "
        .parse()
        .unwrap();
        assert_eq!(
            lookups.get_bishop_attacks(occupied, Square::E4),
            "
                . . . . . . . .
                . . . . . . . #
                . . . . . . # .
                . . . # . # . .
                . . . . . . . .
                . . . # . # . .
                . . # . . . # .
                . . . . . . . #
            "
            .parse()
            .unwrap(),
        );
    }

    #[test]
    fn lookups_get_queen_attacks() {
        let lookups = Lookups::get_instance();
        let occupied = "
                . . . . . . . .
                . . . . . . . .
                . . . . # . . .
                . . . # . . . .
                . . . . # # # .
                . . . # . . . .
                . . # . # . . .
                . . . . . . . .
        "
        .parse()
        .unwrap();
        assert_eq!(
            lookups.get_queen_attacks(occupied, Square::E4),
            "
                . . . . . . . .
                . . . . . . . #
                . . . . # . # .
                . . . # # # . .
                # # # # . # . .
                . . . # # # . .
                . . . . # . # .
                . . . . . . . #
            "
            .parse()
            .unwrap(),
        );
    }

    #[test]
    fn lookups_get_in_between() {
        let lookups = Lookups::get_instance();

        assert_eq!(
            lookups.get_in_between(Square::B4, Square::B8),
            "
                . . . . . . . .
                . # . . . . . .
                . # . . . . . .
                . # . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
            "
            .parse()
            .unwrap()
        );
        assert_eq!(
            lookups.get_in_between(Square::B8, Square::B4),
            "
                . . . . . . . .
                . # . . . . . .
                . # . . . . . .
                . # . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
            "
            .parse()
            .unwrap()
        );

        assert_eq!(
            lookups.get_in_between(Square::B2, Square::H2),
            "
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . . . . . . .
                . . # # # # # .
                . . . . . . . .
            "
            .parse()
            .unwrap()
        );

        assert_eq!(
            lookups.get_in_between(Square::B7, Square::H1),
            "
                . . . . . . . .
                . . . . . . . .
                . . # . . . . .
                . . . # . . . .
                . . . . # . . .
                . . . . . # . .
                . . . . . . # .
                . . . . . . . .
            "
            .parse()
            .unwrap()
        );

        assert_eq!(
            lookups.get_in_between(Square::C3, Square::G1),
            Bitboard::EMPTY
        );
    }
}
