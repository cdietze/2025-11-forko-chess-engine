#[repr(transparent)]
#[derive(Copy, Clone, Default, PartialEq, Eq)]
pub struct BitBoard(pub u64);

use crate::square::{ParseSquareError, Square};
use std::fmt;

impl BitBoard {
    /// Creates a BitBoard with a single bit set at the given square
    #[inline]
    pub const fn from_square(i: Square) -> Self {
        BitBoard(1u64 << i.0)
    }
    #[inline]
    pub const fn from_idx(i: u8) -> Self {
        BitBoard(1u64 << i)
    }

    /// Creates a BitBoard from multiple square indices by combining them with OR operations
    pub fn from_squares(squares: impl IntoIterator<Item = Square>) -> Self {
        squares
            .into_iter()
            .fold(BitBoard::EMPTY, |acc, sq| acc.or(Self::from_square(sq)))
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
        coords.into_iter().try_fold(BitBoard::EMPTY, |acc, s| {
            let sq: Square = s.as_ref().parse()?;
            Ok(acc.or(BitBoard::from_square(sq)))
        })
    }
}

impl fmt::Display for BitBoard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for rank in (0..8).rev() {
            for file in 0..8 {
                let square = rank * 8 + file;
                write!(f, " ")?;
                write!(
                    f,
                    "{}",
                    if (*self).and(BitBoard::from_square(Square(square))) == BitBoard::EMPTY {
                        '.'
                    } else {
                        'x'
                    }
                )?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}
impl fmt::Debug for BitBoard {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Display::fmt(self, f)
    }
}

/// Allow collecting from an iterator of `Square` into a `BitBoard`.
impl FromIterator<Square> for BitBoard {
    fn from_iter<T: IntoIterator<Item = Square>>(iter: T) -> Self {
        BitBoard::from_squares(iter)
    }
}

mod tests {
    use crate::bitboard::BitBoard;

    #[test]
    fn test_print_bitboard() {
        let v: u64 = 282579823558913;
        let bb = BitBoard(v);
        println!("{:?}", bb)
    }
}
