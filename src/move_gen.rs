use crate::bitboard::BitBoard;
use crate::board::{Board, Piece};
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
    let kings = _board.pieces(Piece::King, _board.color_to_move());
    add_king_moves(&mut v, kings);
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
    use crate::board::{Color, Piece};
    use rand::prelude::IndexedRandom;
    use std::collections::HashSet;

    #[test]
    fn generate_moves_white_king_on_a1() {
        let board = Board::from_white_king("a1".parse().unwrap());
        assert_move_sources(&generate_moves(&board), &["a1"]);
        assert_move_destinations(&generate_moves(&board), &["b1", "a2", "b2"]);
    }

    fn assert_move_sources(moves: &[Move], expected: &[&str]) {
        let actual: HashSet<String> = moves.iter().map(|m| m.from().algebraic()).collect();
        let expected: HashSet<String> = expected.iter().map(|&s| s.to_string()).collect();
        assert_eq!(actual, expected);
    }

    fn assert_move_destinations(moves: &[Move], expected: &[&str]) {
        let actual: HashSet<String> = moves.iter().map(|m| m.to().algebraic()).collect();
        let expected: HashSet<String> = expected.iter().map(|&s| s.to_string()).collect();
        assert_eq!(actual, expected);
    }

    #[test]
    fn random_game_two_kings() {
        use rand::SeedableRng;
        use rand::seq::SliceRandom;

        // Create an empty board
        let mut board = Board::empty();
        board.set_piece("e1".parse().unwrap(), Piece::King, Color::White);
        board.set_piece("e8".parse().unwrap(), Piece::King, Color::Black);

        println!("\nInitial position:");
        println!("{}", board);

        let mut rng = rand::rngs::StdRng::seed_from_u64(42);

        // Total of 10 ply moves
        for move_num in 1..=10 {
            let moves = generate_moves(&board);
            println!("Pseudo-Legal Moves: {:?}", moves);
            if let Some(white_move) = moves.choose(&mut rng) {
                println!(
                    "Ply {}: {} plays {} -> {}",
                    move_num,
                    board.color_to_move(),
                    white_move.from().algebraic(),
                    white_move.to().algebraic()
                );
                board.do_move(*white_move);
                println!("{}", board);
            }
        }
    }
}
