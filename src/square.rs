use std::str::FromStr;

#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Square(pub u8);

impl Square {
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
        Square(rank * 8 + file)
    }
}

impl core::fmt::Display for Square {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let idx = self.0;
        let file = (idx % 8) as u8;
        let rank = (idx / 8) as u8;
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

    /**
    Converts given algebraic notation into a Square.
    E.g.
    "a1" -> Square(0)
    "b1" -> Square(1)
    "h8" -> Square(63)
    */
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
