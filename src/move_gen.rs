use crate::bitboard::BitBoard;
use crate::board::SquareIndex;
use crate::bitboard::ParseSquareError;
fn king_attacks(b: BitBoard) -> BitBoard {
    let mut r = b | shift_east(b) | shift_west(b);
    r |= shift_north(r) | shift_south(r);
    r &= !b;
    r
}
#[inline]
fn shift_south(b: BitBoard) -> BitBoard {
    b >> 8u32
}
#[inline]
fn shift_north(b: BitBoard) -> BitBoard {
    b << 8u32
}

#[inline]
fn shift_east(b: BitBoard) -> BitBoard {
    (b << 1u32) & NOT_A_FILE
}
#[inline]
fn shift_west(b: BitBoard) -> BitBoard {
    (b >> 1u32) & NOT_H_FILE
}

const NOT_A_FILE: BitBoard = BitBoard(0xfefefefefefefefe);
const NOT_H_FILE: BitBoard = BitBoard(0x7f7f7f7f7f7f7f7f);
const EMPTY: BitBoard = BitBoard(0);
const FULL: BitBoard = BitBoard(0xffffffffffffffff);

#[inline]
fn single_square(i: SquareIndex) -> BitBoard {
    BitBoard(1u64 << i.0)
}

fn print_bb(b: BitBoard) -> String {
    let mut result = String::new();
    for rank in (0..8).rev() {
        for file in 0..8 {
            let square = rank * 8 + file;
            result.push(' ');
            result.push(if (b & BitBoard(1u64 << square)) == BitBoard(0) {
                '.'
            } else {
                '1'
            });
        }
        result.push('\n');
    }
    result
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bitboard::ParseSquareError;

    #[test]
    fn print_king_attacks_from_g2() {
        let square = "g2";
        let king = BitBoard::from_square(to_index(square).unwrap());
        let attacks = king_attacks(king);
        println!("Input (king on {}):\n{}", square, print_bb(king));
        println!("King attacks from {}:\n{}", square, print_bb(attacks));
    }

    #[test]
    fn test_to_index_corners() {
        assert_eq!(to_index("a1"), Ok(SquareIndex(0)));
        assert_eq!(to_index("h1"), Ok(SquareIndex(7)));
        assert_eq!(to_index("a8"), Ok(SquareIndex(56)));
        assert_eq!(to_index("h8"), Ok(SquareIndex(63)));
    }

    #[test]
    fn test_to_index_examples() {
        assert_eq!(to_index("b1"), Ok(SquareIndex(1)));
        assert_eq!(to_index("g2"), Ok(SquareIndex(14)));
        assert_eq!(to_index("d4"), Ok(SquareIndex(27)));
        assert_eq!(to_index("e5"), Ok(SquareIndex(36)));
    }
    #[test]
    fn test_to_index_negative_examples() {
        assert_eq!(to_index(""), Err(ParseSquareError));
        assert_eq!(to_index("a1a"), Err(ParseSquareError));
        assert_eq!(to_index("1a"), Err(ParseSquareError));
        assert_eq!(to_index("x1"), Err(ParseSquareError));
        assert_eq!(to_index("a0"), Err(ParseSquareError));
        assert_eq!(to_index("a9"), Err(ParseSquareError));
    }
}
