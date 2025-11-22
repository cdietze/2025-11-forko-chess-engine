use std::str::FromStr;

#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Square(pub u8);

impl Square {
    pub const ILLEGAL_SQUARE: Square = Square(64);
    pub fn algebraic(&self) -> String {
        format!("{}", self)
    }
    pub fn file(&self) -> u8 {
        self.0 % 8
    }
    pub fn rank(&self) -> u8 {
        self.0 / 8
    }
    pub fn is_valid(&self) -> bool {
        self.0 < 64
    }
    pub fn from_file_rank(file: u8, rank: u8) -> Square {
        debug_assert!(
            file < 8 && rank < 8,
            "invalid square coordinates ({}, {})",
            file,
            rank
        );
        Square(rank * 8 + file)
    }

    // Indexing: a1 = 0, b1 = 1, ..., h1 = 7, a2 = 8, ..., h8 = 63
    pub const A1: Square = Square(0);
    pub const B1: Square = Square(1);
    pub const C1: Square = Square(2);
    pub const D1: Square = Square(3);
    pub const E1: Square = Square(4);
    pub const F1: Square = Square(5);
    pub const G1: Square = Square(6);
    pub const H1: Square = Square(7);

    pub const A2: Square = Square(8);
    pub const B2: Square = Square(9);
    pub const C2: Square = Square(10);
    pub const D2: Square = Square(11);
    pub const E2: Square = Square(12);
    pub const F2: Square = Square(13);
    pub const G2: Square = Square(14);
    pub const H2: Square = Square(15);

    pub const A3: Square = Square(16);
    pub const B3: Square = Square(17);
    pub const C3: Square = Square(18);
    pub const D3: Square = Square(19);
    pub const E3: Square = Square(20);
    pub const F3: Square = Square(21);
    pub const G3: Square = Square(22);
    pub const H3: Square = Square(23);

    pub const A4: Square = Square(24);
    pub const B4: Square = Square(25);
    pub const C4: Square = Square(26);
    pub const D4: Square = Square(27);
    pub const E4: Square = Square(28);
    pub const F4: Square = Square(29);
    pub const G4: Square = Square(30);
    pub const H4: Square = Square(31);

    pub const A5: Square = Square(32);
    pub const B5: Square = Square(33);
    pub const C5: Square = Square(34);
    pub const D5: Square = Square(35);
    pub const E5: Square = Square(36);
    pub const F5: Square = Square(37);
    pub const G5: Square = Square(38);
    pub const H5: Square = Square(39);

    pub const A6: Square = Square(40);
    pub const B6: Square = Square(41);
    pub const C6: Square = Square(42);
    pub const D6: Square = Square(43);
    pub const E6: Square = Square(44);
    pub const F6: Square = Square(45);
    pub const G6: Square = Square(46);
    pub const H6: Square = Square(47);

    pub const A7: Square = Square(48);
    pub const B7: Square = Square(49);
    pub const C7: Square = Square(50);
    pub const D7: Square = Square(51);
    pub const E7: Square = Square(52);
    pub const F7: Square = Square(53);
    pub const G7: Square = Square(54);
    pub const H7: Square = Square(55);

    pub const A8: Square = Square(56);
    pub const B8: Square = Square(57);
    pub const C8: Square = Square(58);
    pub const D8: Square = Square(59);
    pub const E8: Square = Square(60);
    pub const F8: Square = Square(61);
    pub const G8: Square = Square(62);
    pub const H8: Square = Square(63);
}

impl core::fmt::Display for Square {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let idx = self.0;
        let file = (idx % 8);
        let rank = idx / 8;
        let file_char = (b'a' + file) as char;
        let rank_char = (b'1' + rank) as char;
        write!(f, "{}{}", file_char, rank_char)
    }
}

impl core::fmt::Debug for Square {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // Use Display format for Debug too, so test output shows algebraic notation
        core::fmt::Display::fmt(self, f)
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

impl FromStr for Square {
    type Err = ParseSquareError;

    /// Converts algebraic notation ("a1", "b1", ... "h8") into a `Square` (0, 1, ... 63).
    fn from_str(coords: &str) -> Result<Self, Self::Err> {
        let b = coords.as_bytes();
        if b.len() != 2 {
            return Err(ParseSquareError);
        }
        let file = match b[0] {
            b'a'..=b'h' => b[0] - b'a',
            b'A'..=b'H' => b[0] - b'A',
            _ => return Err(ParseSquareError),
        };
        let rank = match b[1] {
            b'1'..=b'8' => b[1] - b'1',
            _ => return Err(ParseSquareError),
        };
        Ok(Square(rank * 8 + file))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::square::ParseSquareError;

    #[test]
    fn parse_algebraic_notation() {
        assert_eq!("a1".parse(), Ok(Square(0)));
        assert_eq!("h1".parse(), Ok(Square(7)));
        assert_eq!("a8".parse(), Ok(Square(56)));
        assert_eq!("h8".parse(), Ok(Square(63)));
        assert_eq!("b1".parse(), Ok(Square(1)));
        assert_eq!("g2".parse(), Ok(Square(14)));
        assert_eq!("d4".parse(), Ok(Square(27)));
        assert_eq!("e5".parse(), Ok(Square(36)));
    }
    #[test]
    fn parse_algebraic_notation_negative_examples() {
        assert_eq!("".parse::<Square>(), Err(ParseSquareError));
        assert_eq!("a1a".parse::<Square>(), Err(ParseSquareError));
        assert_eq!("1a".parse::<Square>(), Err(ParseSquareError));
        assert_eq!("x1".parse::<Square>(), Err(ParseSquareError));
        assert_eq!("a0".parse::<Square>(), Err(ParseSquareError));
        assert_eq!("a9".parse::<Square>(), Err(ParseSquareError));
    }
}
