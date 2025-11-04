use crate::bitboard::ParseSquareError;
use std::str::FromStr;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Square(pub u8);

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
