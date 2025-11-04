#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct BitBoard(pub u64);

impl BitBoard {
    /// Creates a BitBoard with a single bit set at the given square
    #[inline]
    pub fn from_square(i: SquareIndex) -> Self {
        BitBoard(1u64 << i.0)
    }
    /// Creates a BitBoard from multiple square indices by combining them with OR operations
    #[inline]
    pub fn from_squares(squares: impl IntoIterator<Item = SquareIndex>) -> Self {
        squares
            .into_iter()
            .fold(BitBoard(0), |acc, sq| acc | Self::from_square(sq))
    }

    /// Tries to create a BitBoard from coordinate strings like "a1", "e5", "f3".
    ///
    /// Uses the existing coordinate-to-index conversion and returns an error
    /// if any coordinate is invalid.
    ///
    /// Examples:
    /// let bb = BitBoard::try_from_coords(["a1", "e5", "f3"])?;
    /// let bb2 = BitBoard::try_from_coords(vec!["b2".to_string(), "h8".to_string()])?;
    #[inline]
    pub fn try_from_coords<S, I>(coords: I) -> Result<Self, ParseSquareError>
    where
        S: AsRef<str>,
        I: IntoIterator<Item = S>,
    {
        let mut acc = BitBoard(0);
        for s in coords {
            let sq = crate::move_gen::to_index(s.as_ref())
                .ok_or(ParseSquareError)?;
            acc |= Self::from_square(sq);
        }
        Ok(acc)
    }
}

/// Error type for parsing algebraic square coordinates (like "a1").
/// 
/// This is kept intentionally lightweight as only a single failure mode is needed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParseSquareError;

impl core::fmt::Display for ParseSquareError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "invalid coordinate (expected like \"e4\")")
    }
}

/// Allow collecting from an iterator of `SquareIndex` into a `BitBoard`.
impl core::iter::FromIterator<crate::board::SquareIndex> for BitBoard {
    fn from_iter<T: IntoIterator<Item = crate::board::SquareIndex>>(iter: T) -> Self {
        BitBoard::from_squares(iter)
    }
}

// Bitwise and shift operators so we can write idiomatic expressions on BitBoard
use core::ops::{
    BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not, Shl, ShlAssign, Shr,
    ShrAssign,
};
use crate::board::SquareIndex;

impl BitAnd for BitBoard {
    type Output = Self;
    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        BitBoard(self.0 & rhs.0)
    }
}
impl BitAndAssign for BitBoard {
    #[inline]
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}
impl BitOr for BitBoard {
    type Output = Self;
    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        BitBoard(self.0 | rhs.0)
    }
}
impl BitOrAssign for BitBoard {
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}
impl BitXor for BitBoard {
    type Output = Self;
    #[inline]
    fn bitxor(self, rhs: Self) -> Self::Output {
        BitBoard(self.0 ^ rhs.0)
    }
}
impl BitXorAssign for BitBoard {
    #[inline]
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0;
    }
}
impl Not for BitBoard {
    type Output = Self;
    #[inline]
    fn not(self) -> Self::Output {
        BitBoard(!self.0)
    }
}

// Shifts: implement for u32 and u8 to make using integer literals ergonomic
impl Shl<u32> for BitBoard {
    type Output = Self;
    #[inline]
    fn shl(self, rhs: u32) -> Self::Output {
        BitBoard(self.0 << rhs)
    }
}
impl Shr<u32> for BitBoard {
    type Output = Self;
    #[inline]
    fn shr(self, rhs: u32) -> Self::Output {
        BitBoard(self.0 >> rhs)
    }
}
impl Shl<u8> for BitBoard {
    type Output = Self;
    #[inline]
    fn shl(self, rhs: u8) -> Self::Output {
        BitBoard(self.0 << rhs)
    }
}
impl Shr<u8> for BitBoard {
    type Output = Self;
    #[inline]
    fn shr(self, rhs: u8) -> Self::Output {
        BitBoard(self.0 >> rhs)
    }
}
impl ShlAssign<u32> for BitBoard {
    #[inline]
    fn shl_assign(&mut self, rhs: u32) {
        self.0 <<= rhs;
    }
}
impl ShrAssign<u32> for BitBoard {
    #[inline]
    fn shr_assign(&mut self, rhs: u32) {
        self.0 >>= rhs;
    }
}
impl ShlAssign<u8> for BitBoard {
    #[inline]
    fn shl_assign(&mut self, rhs: u8) {
        self.0 <<= rhs;
    }
}
impl ShrAssign<u8> for BitBoard {
    #[inline]
    fn shr_assign(&mut self, rhs: u8) {
        self.0 >>= rhs;
    }
}
