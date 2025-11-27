use crate::board::Piece;
use crate::precomputed::CastleSide;
use crate::precomputed::CastleSide::KingSide;
use crate::square::Square;

/// Encodes a chess move into a compact 16-bit value.
///
/// Bit layout (little-endian within the 16-bit integer):
/// - bits 0..=5   (6 bits): from-square index (0..=63)
/// - bits 6..=11  (6 bits): to-square index (0..=63)
/// - bit 12: promotion flag
/// - bit 13: capture flag
/// - bits 14..=15 (2 bits): special flags
///
/// Promotion piece encoding in bits 14..=15 when the promotion flag (bit 12) is set:
/// - 00 = knight, 01 = bishop, 10 = rook, 11 = queen.
///
/// Adapted from: https://www.chessprogramming.org/Encoding_Moves#From-To_Based

#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Move(u16);

impl Move {
    const FROM_BITS: u16 = 6;
    const TO_BITS: u16 = 6;
    const FLAG_BITS: u16 = 4; // 16 - 6 - 6

    const FROM_SHIFT: u16 = 0;
    const TO_SHIFT: u16 = Self::FROM_SHIFT + Self::FROM_BITS; // 6
    const FLAG_SHIFT: u16 = Self::TO_SHIFT + Self::TO_BITS; // 12

    const PROMOTION_SHIFT: u16 = Self::FLAG_SHIFT;
    const CAPTURE_SHIFT: u16 = Self::FLAG_SHIFT + 1;
    const SPECIAL_0_SHIFT: u16 = Self::CAPTURE_SHIFT + 1;
    const SPECIAL_1_SHIFT: u16 = Self::SPECIAL_0_SHIFT + 1;

    const FROM_MASK: u16 = (1u16 << Self::FROM_BITS) - 1; // 0x3F
    const TO_MASK: u16 = (1u16 << Self::TO_BITS) - 1; // 0x3F
    const FLAG_MASK: u16 = (1u16 << Self::FLAG_BITS) - 1;

    #[inline]
    pub const fn new(
        from: Square,
        to: Square,
        promotion: bool,
        capture: bool,
        special0: bool,
        special1: bool,
    ) -> Self {
        let f = (from.0 as u16) & Self::FROM_MASK;
        let t = (to.0 as u16) & Self::TO_MASK;
        let flags = ((promotion as u16) << 0)
            | ((capture as u16) << 1)
            | ((special0 as u16) << 2)
            | ((special1 as u16) << 3);
        Self(
            (f << Self::FROM_SHIFT)
                | (t << Self::TO_SHIFT)
                | ((flags & Self::FLAG_MASK) << Self::FLAG_SHIFT),
        )
    }

    #[inline]
    pub const fn new_quiet(from: Square, to: Square) -> Self {
        Self::new(from, to, false, false, false, false)
    }

    #[inline]
    pub const fn new_capture(from: Square, to: Square) -> Self {
        Self::new(from, to, false, true, false, false)
    }

    #[inline]
    pub const fn new_double_pawn_push(from: Square, to: Square) -> Self {
        Self::new(from, to, false, false, false, true)
    }

    #[inline]
    pub const fn new_promotion(from: Square, to: Square, capture: bool, piece: Piece) -> Self {
        debug_assert!(
            matches!(
                piece,
                Piece::Knight | Piece::Bishop | Piece::Rook | Piece::Queen
            ),
            "promotion piece must be Knight/Bishop/Rook/Queen"
        );
        let (special0, special1) = match piece {
            Piece::Knight => (false, false),
            Piece::Bishop => (true, false),
            Piece::Rook => (false, true),
            Piece::Queen => (true, true),
            _ => (true, true), // Fallback in release builds
        };
        Self::new(from, to, true, capture, special0, special1)
    }

    #[inline]
    pub const fn new_en_passant(from: Square, to: Square) -> Self {
        Self::new(from, to, false, true, false, true)
    }

    #[inline]
    pub const fn new_castle(from: Square, to: Square, castle_side: CastleSide) -> Self {
        let (special0, special1) = match castle_side {
            KingSide => (true, false),
            _ => (true, true),
        };
        Self::new(from, to, false, false, special0, special1)
    }

    /// Returns the source square.
    #[inline]
    pub const fn from(self) -> Square {
        Square(((self.0 >> Self::FROM_SHIFT) & Self::FROM_MASK) as u8)
    }

    /// Returns the destination square.
    #[inline]
    pub const fn to(self) -> Square {
        Square(((self.0 >> Self::TO_SHIFT) & Self::TO_MASK) as u8)
    }

    #[inline]
    pub const fn promotion(self) -> bool {
        (self.0 & (1u16 << Self::PROMOTION_SHIFT)) != 0
    }
    #[inline]
    pub const fn special0(self) -> bool {
        (self.0 & (1u16 << Self::SPECIAL_0_SHIFT)) != 0
    }

    #[inline]
    pub const fn special1(self) -> bool {
        (self.0 & (1u16 << Self::SPECIAL_1_SHIFT)) != 0
    }

    /// If this is a promotion move, returns the encoded piece to promote to.
    pub const fn promotion_piece(self) -> Piece {
        match (self.0 >> 14) & 0b11 {
            0b00 => Piece::Knight,
            0b01 => Piece::Bishop,
            0b10 => Piece::Rook,
            0b11 => Piece::Queen,
            _ => unreachable!(),
        }
    }

    #[inline]
    pub const fn capture(self) -> bool {
        (self.0 & (1u16 << Self::CAPTURE_SHIFT)) != 0
    }

    pub fn algebraic(self) -> String {
        self.to_string()
    }
}

/// Error type for parsing algebraic move coordinates like "e2e4".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParseMoveError;

impl core::fmt::Display for ParseMoveError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "invalid move coordinate (expected like \"e2e4\")")
    }
}

impl core::fmt::Display for Move {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.promotion() {
            let letter = match self.promotion_piece() {
                Piece::Knight => 'N',
                Piece::Bishop => 'B',
                Piece::Rook => 'R',
                Piece::Queen => 'Q',
                _ => '?',
            };
            write!(f, "{}{}={}", self.from(), self.to(), letter)
        } else {
            write!(f, "{}{}", self.from(), self.to())
        }
    }
}

impl core::fmt::Debug for Move {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // Use Display format for Debug too for concise output
        core::fmt::Display::fmt(self, f)
    }
}

impl std::str::FromStr for Move {
    type Err = ParseMoveError;

    fn from_str(coords: &str) -> Result<Self, Self::Err> {
        // Delegate to Square::from_str for both squares to avoid duplicate parsing logic.
        let from = coords.get(0..2).ok_or(ParseMoveError).and_then(|s| {
            <crate::square::Square as std::str::FromStr>::from_str(s).map_err(|_| ParseMoveError)
        })?;
        let to = coords.get(2..4).ok_or(ParseMoveError).and_then(|s| {
            <crate::square::Square as std::str::FromStr>::from_str(s).map_err(|_| ParseMoveError)
        })?;

        // Optional promotion suffix: e.g., "e7e8=Q"
        let rest = coords.get(4..).unwrap_or("");
        if rest.is_empty() {
            return Ok(Self::new_quiet(from, to));
        }

        // Expect exactly "=<piece>" where piece is one of N, B, R, Q (case-insensitive)
        if !rest.starts_with('=') || rest.chars().count() != 2 {
            return Err(ParseMoveError);
        }
        let letter = rest.chars().nth(1).unwrap();
        let piece = match letter.to_ascii_uppercase() {
            'N' => Piece::Knight,
            'B' => Piece::Bishop,
            'R' => Piece::Rook,
            'Q' => Piece::Queen,
            _ => return Err(ParseMoveError),
        };

        Ok(Self::new_promotion(from, to, false, piece))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_basic() {
        let from = Square(0);
        let to = Square(63);
        let m = Move::new_quiet(from, to);
        assert_eq!(m.from(), Square(0));
        assert_eq!(m.to(), Square(63));
        assert_eq!(m.special0(), false);
        assert_eq!(m.special1(), false);
    }

    #[test]
    fn parse_from_coords_str() {
        use std::str::FromStr;
        let m = Move::from_str("e2e4").expect("valid coords");
        assert_eq!(m.from(), Square(12)); // e2 => 1*8 + 4 = 12
        assert_eq!(m.to(), Square(28)); // e4 => 3*8 + 4 = 28
        assert_eq!(m.special0(), false);
        assert_eq!(m.special1(), false);
    }

    #[test]
    fn from_str_trait_works() {
        use std::str::FromStr;
        let m = Move::from_str("b1c3").unwrap();
        assert_eq!(m.from(), Square(1)); // b1 => 0*8 + 1
        assert_eq!(m.to(), Square(18)); // c3 => 2*8 + 2
        assert_eq!(m.special0(), false);
        assert_eq!(m.special1(), false);
    }
    #[test]
    fn promotion_and_capture_bits_work() {
        let from = Square(12); // arbitrary
        let to = Square(28);
        // Base quiet move
        let m0 = Move::new(from, to, false, false, false, false);
        assert!(!m0.promotion());
        assert!(!m0.capture());
        // Capture only
        let mc = Move::new(from, to, false, true, false, false);
        assert!(!mc.promotion());
        assert!(mc.capture());
        // Promotion only
        let mp = Move::new(from, to, true, false, false, false);
        assert!(mp.promotion());
        assert!(!mp.capture());
        // Both promotion and capture
        let mpc = Move::new(from, to, true, true, false, false);
        assert!(mpc.promotion());
        assert!(mpc.capture());
    }
    #[test]
    fn double_pawn_push_is_not_capture() {
        let from = Square(8); // e.g., a white pawn from rank 2
        let to = Square(24); // pushed two squares
        let m = Move::new_double_pawn_push(from, to);
        assert!(!m.capture());
        assert!(!m.promotion());
    }
}
