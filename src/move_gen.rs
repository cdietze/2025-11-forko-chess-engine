use crate::bitboard::BitBoard;
use crate::square::Square;

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
        let king = BitBoard::from_square(square.parse().unwrap());
        let attacks = king_attacks(king);
        println!("Input (king on {}):\n{}", square, king);
        println!("King attacks from {}:\n{}", square, attacks);
    }

    #[test]
    fn test_to_index_corners() {
        assert_eq!("a1".parse(), Ok(Square(0)));
        assert_eq!("h1".parse(), Ok(Square(7)));
        assert_eq!("a8".parse(), Ok(Square(56)));
        assert_eq!("h8".parse(), Ok(Square(63)));
    }

    #[test]
    fn test_to_index_examples() {
        assert_eq!("b1".parse(), Ok(Square(1)));
        assert_eq!("g2".parse(), Ok(Square(14)));
        assert_eq!("d4".parse(), Ok(Square(27)));
        assert_eq!("e5".parse(), Ok(Square(36)));
    }
    #[test]
    fn test_to_index_negative_examples() {
        let parse = |s: &str| s.parse::<Square>();
        assert_eq!("".parse::<Square>(), Err(ParseSquareError));
        assert_eq!("a1a".parse::<Square>(), Err(ParseSquareError));
        assert_eq!("1a".parse::<Square>(), Err(ParseSquareError));
        assert_eq!("x1".parse::<Square>(), Err(ParseSquareError));
        assert_eq!("a0".parse::<Square>(), Err(ParseSquareError));
        assert_eq!("a9".parse::<Square>(), Err(ParseSquareError));
    }
}
