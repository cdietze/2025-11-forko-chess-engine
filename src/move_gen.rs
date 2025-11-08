use crate::bitboard::BitBoard;
use crate::board::{Board, Piece};
use crate::r#move::Move;
use crate::precomputed::{KING_MOVES, RAYS, Rays};
use crate::square::Square;
use std::array;

impl BitBoard {}

/// Generates a list of pseudo-legal moves from given board.
pub fn generate_moves(board: &Board) -> Vec<Move> {
    let mut v = Vec::new();

    let own_color_board: BitBoard = if board.white_to_move {
        board.white
    } else {
        board.white.not()
    };
    let occupied = board.occupied();

    let add_king_moves = |v: &mut Vec<Move>, b: BitBoard| {
        b.for_each_set_bit(|square| {
            let moves = KING_MOVES[square.0 as usize];
            // Don't capture own pieces
            let moves = moves.and(occupied.and(own_color_board).not());
            moves.for_each_set_bit(|move_square| v.push(Move::new(square, move_square)))
        });
    };

    let kings = board.pieces(Piece::King, board.color_to_move());
    add_king_moves(&mut v, kings);
    let rooks = board.pieces(Piece::Rook, board.color_to_move());
    rooks.for_each_set_bit(|rook_square| {
        add_rook_moves(&mut v, board, rook_square.0);
    });
    v
}

fn add_rook_moves(v: &mut Vec<Move>, board: &Board, square: u8) {
    let moves = rook_moves(board, square);
    moves.for_each_set_bit(|move_square| v.push(Move::new(Square(square), move_square)))
}

fn rook_moves(board: &Board, square: u8) -> BitBoard {
    let rays = RAYS[square as usize];
    let occupied = board.occupied();
    let own_color_board: BitBoard = if board.white_to_move {
        board.white
    } else {
        board.white.not()
    };

    // Helper: trim a ray by the first blocker in that direction, keeping the blocker square
    // Uses bit_scan_forward for directions with increasing indices (north, east)
    // and bit_scan_backward for decreasing indices (south, west).
    let mut trim = |mut ray: BitBoard, dir: fn(&Rays) -> BitBoard, forward: bool| -> BitBoard {
        let blockers = ray.and(occupied);
        if !blockers.is_empty() {
            let b = if forward {
                blockers.bit_scan_forward()
            } else {
                blockers.bit_scan_backward()
            };
            let mask_beyond = dir(&RAYS[b as usize]);
            ray = ray.and(mask_beyond.not());
        }
        ray
    };

    // Compute per-direction trimmed rays
    let north = trim(rays.north, |r| r.north, true);
    let south = trim(rays.south, |r| r.south, false);
    let east = trim(rays.east, |r| r.east, true);
    let west = trim(rays.west, |r| r.west, false);

    // Combine
    let rays = north.or(south).or(east).or(west);
    // Remove blockers of own color
    let o = occupied.and(own_color_board).not();
    let rays = rays.and(o);
    rays
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::{Color, Piece};
    use rand::prelude::IndexedRandom;
    use std::collections::BTreeSet;

    #[test]
    fn king_should_move_correct() {
        let board = Board::empty().set_piece("a1".parse().unwrap(), Piece::King, Color::White);
        assert_move_sources(&generate_moves(&board), &["a1"]);
        assert_move_destinations(&generate_moves(&board), &["b1", "a2", "b2"]);
    }

    #[test]
    fn king_should_try_to_capture_opponents_piece() {
        let board = Board::empty()
            .set_piece("a1".parse().unwrap(), Piece::King, Color::White)
            .set_piece("a2".parse().unwrap(), Piece::Pawn, Color::Black);
        assert_move_destinations(&generate_moves(&board), &["a2", "b1", "b2"]);
    }

    #[test]
    fn king_should_not_capture_own_piece() {
        let board = Board::empty()
            .set_piece("a1".parse().unwrap(), Piece::King, Color::White)
            .set_piece("a2".parse().unwrap(), Piece::Pawn, Color::White);
        assert_move_destinations(&generate_moves(&board), &["b1", "b2"]);
    }
    #[test]
    fn rook_should_move_correctly_from_a1() {
        let board = Board::empty().set_piece("a1".parse().unwrap(), Piece::Rook, Color::White);
        let moves = generate_moves(&board);
        assert_move_sources(&moves, &["a1"]);
        assert_move_destinations(
            &moves,
            &[
                "a2", "a3", "a4", "a5", "a6", "a7", "a8", "b1", "c1", "d1", "e1", "f1", "g1", "h1",
            ],
        );
    }
    #[test]
    fn rook_should_move_correctly_from_e4() {
        let board = Board::empty().set_piece("e4".parse().unwrap(), Piece::Rook, Color::White);
        let moves = generate_moves(&board);
        assert_move_sources(&moves, &["e4"]);
        assert_move_destinations(
            &moves,
            &[
                "e1", "e2", "e3", "e5", "e6", "e7", "e8", "a4", "b4", "c4", "d4", "f4", "g4", "h4",
            ],
        );
    }

    #[test]
    fn black_rook_should_move_correctly_from_e4() {
        let board = Board::empty()
            .set_piece("e4".parse().unwrap(), Piece::Rook, Color::Black)
            .set_color_to_move(Color::Black);
        let moves = generate_moves(&board);
        assert_move_sources(&moves, &["e4"]);
        assert_move_destinations(
            &moves,
            &[
                "e1", "e2", "e3", "e5", "e6", "e7", "e8", "a4", "b4", "c4", "d4", "f4", "g4", "h4",
            ],
        );
    }

    #[test]
    fn rook_should_move_correctly_with_blockers() {
        let board = Board::empty()
            .set_piece("a4".parse().unwrap(), Piece::Rook, Color::White)
            .set_piece("a7".parse().unwrap(), Piece::Pawn, Color::Black)
            .set_piece("f4".parse().unwrap(), Piece::Pawn, Color::White)
            .set_piece("f5".parse().unwrap(), Piece::Pawn, Color::Black);
        let moves = generate_moves(&board);
        assert_move_sources(&moves, &["a4"]);
        assert_move_destinations(
            &moves,
            &["a1", "a2", "a3", "a5", "a6", "a7", "b4", "c4", "d4", "e4"],
        );
    }

    fn assert_move_sources(moves: &[Move], expected: &[&str]) {
        let actual: BTreeSet<String> = moves.iter().map(|m| m.from().algebraic()).collect();
        let expected: BTreeSet<String> = expected.iter().map(|&s| s.to_string()).collect();
        assert_eq!(actual, expected);
    }

    fn assert_move_destinations(moves: &[Move], expected: &[&str]) {
        let actual: BTreeSet<String> = moves.iter().map(|m| m.to().algebraic()).collect();
        let expected: BTreeSet<String> = expected.iter().map(|&s| s.to_string()).collect();
        assert_eq!(actual, expected);
    }

    #[test]
    fn random_game_two_kings() {
        use rand::SeedableRng;

        let mut board = Board::empty()
            .set_piece("e1".parse().unwrap(), Piece::King, Color::White)
            .set_piece("e8".parse().unwrap(), Piece::King, Color::Black);

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
