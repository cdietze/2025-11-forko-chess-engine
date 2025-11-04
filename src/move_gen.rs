use crate::bitboard::BitBoard;
use crate::square::{SquareIndex, to_index};

fn king_attacks(b: BitBoard) -> BitBoard {
    let mut r = b | b.shift_east() | b.shift_west();
    r |= r.shift_north() | r.shift_south();
    r &= !b;
    r
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
        println!("Input (king on {}):\n{}", square, king);
        println!("King attacks from {}:\n{}", square, attacks);
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
