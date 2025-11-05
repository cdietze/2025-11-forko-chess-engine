use crate::bitboard::BitBoard;
use crate::board::Board;
use crate::r#move::Move;
use crate::square::Square;

impl BitBoard {
    const fn king_moves(self) -> BitBoard {
        let b = self;
        let mut r = b.or(b.shift_east()).or(b.shift_west());
        r = r.or(r.shift_north()).or(r.shift_south());
        r.and(b.not())
    }
}

/// Generates a list of pseudo-legal moves from given board.
pub fn generate_moves(_board: &Board) -> Vec<Move> {
    let mut v = Vec::new();
    add_king_moves(&mut v, _board.white_kings());
    v
}

fn add_king_moves(v: &mut Vec<Move>, b: BitBoard) {
    b.for_each_set_bit(|square| {
        let moves = KING_MOVES[square.0 as usize];
        moves.for_each_set_bit(|move_square| v.push(Move::new(square, move_square)))
    });
}

/// Precomputed king move bitboards
const KING_MOVES: [BitBoard; 64] = {
    let mut arr = [BitBoard::EMPTY; 64];
    let mut i = 0;
    while i < 64 {
        let bb = BitBoard::from_square(Square(i as u8));
        arr[i] = bb.king_moves();
        i += 1;
    }
    arr
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::square::ParseSquareError;
    use std::collections::HashSet;

    #[test]
    fn print_king_attacks_from_a1() {
        let square = "a1";
        let king = BitBoard::from_square(square.parse().unwrap());
        let attacks = king.king_moves();
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
        assert_eq!("".parse::<Square>(), Err(ParseSquareError));
        assert_eq!("a1a".parse::<Square>(), Err(ParseSquareError));
        assert_eq!("1a".parse::<Square>(), Err(ParseSquareError));
        assert_eq!("x1".parse::<Square>(), Err(ParseSquareError));
        assert_eq!("a0".parse::<Square>(), Err(ParseSquareError));
        assert_eq!("a9".parse::<Square>(), Err(ParseSquareError));
    }

    #[test]
    fn generate_moves_white_king_on_a1() {
        let board = Board::from_white_king("a1".parse().unwrap());
        assert_move_destinations(&generate_moves(&board), &["b1", "a2", "b2"]);
    }

    fn assert_move_destinations(moves: &[Move], expected: &[&str]) {
        let actual: HashSet<String> = moves.iter().map(|m| m.to().algebraic()).collect();
        let expected: HashSet<String> = expected.iter().map(|&s| s.to_string()).collect();
        assert_eq!(actual, expected);
    }
}
