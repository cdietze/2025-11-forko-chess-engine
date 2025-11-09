use crate::square::Square;

/// Encodes a chess move into a compact 16-bit value.
///
/// Bit layout (little-endian within the 16-bit integer):
/// - bits 0..=5   (6 bits): from-square index (0..=63)
/// - bits 6..=11  (6 bits): to-square index (0..=63)
/// - bits 12..=15 (4 bits): flags (free-form; can encode capture, castle, promotion kind, etc.)
///
/// Notes and guarantees:
/// - All constructors mask their inputs, so out-of-range values are truncated to the valid bit width
///   instead of panicking. This makes the API fast and constexpr-friendly.
/// - Accessors (`from`, `to`, `flags`) always return masked values in the valid range.
/// - The type is a `#[repr(transparent)]` wrapper over `u16` for zero-cost passing and copying.
///
/// Example
/// ```text
/// use cpd_chess::square::Square;
/// use cpd_chess::mv::Move; // re-export path may differ depending on your module setup
///
/// let m = Move::from_parts(0, 63, 0b1010);
/// assert_eq!(m.from(), Square(0));
/// assert_eq!(m.to(), Square(63));
/// assert_eq!(m.flags(), 0b1010);
/// ```

#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Move(u16);

impl Move {
    const FROM_BITS: u16 = 6;
    const TO_BITS: u16 = 6;
    const FLAG_BITS: u16 = 4; // 16 - 6 - 6

    const FROM_SHIFT: u16 = 0;
    const TO_SHIFT: u16 = Self::FROM_SHIFT + Self::FROM_BITS; // 6
    const FLAG_SHIFT: u16 = Self::TO_SHIFT + Self::TO_BITS; // 12

    const FROM_MASK: u16 = (1u16 << Self::FROM_BITS) - 1; // 0x3F
    const TO_MASK: u16 = (1u16 << Self::TO_BITS) - 1; // 0x3F
    const FLAG_MASK: u16 = (1u16 << Self::FLAG_BITS) - 1; // 0x0F

    /// Creates a move from two squares. Extra bits in the inputs are masked.
    #[inline]
    pub const fn new(from: Square, to: Square) -> Self {
        let f = (from.0 as u16) & Self::FROM_MASK;
        let t = (to.0 as u16) & Self::TO_MASK;
        Self((f << Self::FROM_SHIFT) | (t << Self::TO_SHIFT))
    }

    /// Creates a move from raw indices (0..=63). Extra bits are masked.
    #[inline]
    pub const fn from_indices(from: u8, to: u8) -> Self {
        let f = (from as u16) & Self::FROM_MASK;
        let t = (to as u16) & Self::TO_MASK;
        Self((f << Self::FROM_SHIFT) | (t << Self::TO_SHIFT))
    }

    /// Creates a move from parts (indices + flags). Extra bits are masked.
    #[inline]
    pub const fn from_parts(from: u8, to: u8, flags: u8) -> Self {
        let base = Self::from_indices(from, to).0;
        let fl = ((flags as u16) & Self::FLAG_MASK) << Self::FLAG_SHIFT;
        Self(base | fl)
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

    /// Returns the 4-bit flags field.
    #[inline]
    pub const fn flags(self) -> u8 {
        ((self.0 >> Self::FLAG_SHIFT) & Self::FLAG_MASK) as u8
    }

    /// Returns the underlying 16-bit representation.
    #[inline]
    pub const fn raw(self) -> u16 {
        self.0
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
        write!(f, "{}{}", self.from(), self.to())
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
        Ok(Move::new(from, to))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_basic() {
        let m = Move::new(Square(0), Square(63));
        assert_eq!(m.from(), Square(0));
        assert_eq!(m.to(), Square(63));
        assert_eq!(m.flags(), 0);
    }

    #[test]
    fn from_parts_works() {
        let m = Move::from_parts(7, 56, 0b1111);
        assert_eq!(m.from(), Square(7));
        assert_eq!(m.to(), Square(56));
        assert_eq!(m.flags(), 0b1111);
        assert_eq!(m.raw() & 0xFFFF, m.raw());
    }

    #[test]
    fn parse_from_coords_str() {
        use std::str::FromStr;
        let m = Move::from_str("e2e4").expect("valid coords");
        assert_eq!(m.from(), Square(12)); // e2 => 1*8 + 4 = 12
        assert_eq!(m.to(), Square(28)); // e4 => 3*8 + 4 = 28
        assert_eq!(m.flags(), 0);
    }

    #[test]
    fn from_str_trait_works() {
        use std::str::FromStr;
        let m = Move::from_str("b1c3").unwrap();
        assert_eq!(m.from(), Square(1)); // b1 => 0*8 + 1
        assert_eq!(m.to(), Square(18)); // c3 => 2*8 + 2
        assert_eq!(m.flags(), 0);
    }
}
