use crate::square::Square;
use std::fmt;

#[repr(transparent)]
#[derive(Copy, Clone, Default, PartialEq, Eq)]
pub struct BitBoard(pub u64);

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
    pub const fn from_squares(squares: &[Square]) -> Self {
        let mut acc = BitBoard::EMPTY;
        let mut i = 0;
        while i < squares.len() {
            acc = acc.or(Self::from_square(squares[i]));
            i += 1;
        }
        acc
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
        // Start on a fresh line in debug output (used e.g. by assert_eq!)
        writeln!(f)?;
        core::fmt::Display::fmt(self, f)
    }
}

#[cfg(test)]
mod tests {
    use super::BitBoard;

    #[test]
    fn test_print_bitboard() {
        let v: u64 = 282579823558913;
        let bb = BitBoard(v);
        println!("{:?}", bb)
    }
}
