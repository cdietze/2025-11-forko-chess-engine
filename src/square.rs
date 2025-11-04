use crate::bitboard::ParseSquareError;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct SquareIndex(pub u8);

/**
Converts given `coords` into its index on the board.
E.g.
"a1" -> 0
"b1" -> 1
"h8" -> 63
*/
pub fn to_index(coords: &str) -> Result<SquareIndex, ParseSquareError> {
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
    Ok(SquareIndex(rank * 8 + file))
}
