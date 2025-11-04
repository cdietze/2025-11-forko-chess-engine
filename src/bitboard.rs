#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct BitBoard(pub u64);

use std::fmt;
use crate::square::SquareIndex;

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
    /// Returns an error if any coordinate is invalid.
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
            let sq = crate::square::to_index(s.as_ref())?;
            acc |= Self::from_square(sq);
        }
        Ok(acc)
    }
}

impl fmt::Display for BitBoard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for rank in (0..8).rev() {
            for file in 0..8 {
                let square = rank * 8 + file;
                write!(f, " ")?;
                write!(f, "{}",
                       if (*self & BitBoard(1u64 << square)) == BitBoard(0) {
                           '.'
                       } else {
                           '1'
                       }
                )?;
            }
            writeln!(f)?;
        }
        Ok(())
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
impl core::iter::FromIterator<crate::square::SquareIndex> for BitBoard {
    fn from_iter<T: IntoIterator<Item = crate::square::SquareIndex>>(iter: T) -> Self {
        BitBoard::from_squares(iter)
    }
}
