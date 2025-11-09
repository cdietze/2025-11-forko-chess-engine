use crate::bitboard::BitBoard;
use crate::board::{Board, Color, Piece};
use crate::r#move::Move;
use crate::precomputed::{KING_MOVES, RAYS, Rays};
use crate::square::Square;

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

    let opponent_attack_map = generate_opponent_attack_map(board);

    board
        .pieces(Piece::King, board.color_to_move())
        .for_each_set_bit(|king_square| {
            let tos = KING_MOVES[king_square.0 as usize];
            // Don't capture own pieces
            let tos = tos.and(occupied.and(own_color_board).not());
            // Don't move into check
            let tos = tos.and(opponent_attack_map.not());
            tos.for_each_set_bit(|to_square| v.push(Move::new(king_square, to_square)))
        });

    for_each_sliding_piece(board, board.color_to_move(), |square, b| {
        b.for_each_set_bit(|to_square| {
            v.push(Move::new(square, to_square));
        })
    });
    v
}

fn for_each_sliding_piece(board: &Board, color: Color, mut f: impl FnMut(Square, BitBoard)) {
    board
        .pieces(Piece::Rook, color)
        .for_each_set_bit(|square| f(square, rook_moves(board, square)));
    board
        .pieces(Piece::Bishop, color)
        .for_each_set_bit(|square| f(square, bishop_moves(board, square)));
    board
        .pieces(Piece::Queen, color)
        .for_each_set_bit(|square| f(square, queen_moves(board, square)));
}

fn generate_opponent_attack_map(board: &Board) -> BitBoard {
    generate_attack_map(board, board.color_to_move().opposite())
}
/// Generates an attack map for the specified color on the given chess board.
///
/// This function evaluates the board state to determine all the squares that are
/// currently being attacked by the specified color's pieces. The attack map is a
/// `BitBoard`, where each set bit denotes a square that is under attack by the given color.
fn generate_attack_map(board: &Board, color: Color) -> BitBoard {
    let mut map = BitBoard::EMPTY;
    board
        .pieces(Piece::King, color)
        .for_each_set_bit(|king_square| {
            map = map.or(KING_MOVES[king_square.0 as usize]);
        });
    board
        .pieces(Piece::Rook, color)
        .for_each_set_bit(|rook_square| {
            map = map.or(rook_moves(board, rook_square));
        });
    map
}

fn sliding_moves(board: &Board, square: Square, orthogonal: bool, diagonal: bool) -> BitBoard {
    let rays = RAYS[square.0 as usize];
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

    let mut result = BitBoard::EMPTY;
    // Compute per-direction trimmed rays
    if orthogonal {
        result = result.or(trim(rays.north, |r| r.north, true));
        result = result.or(trim(rays.south, |r| r.south, false));
        result = result.or(trim(rays.east, |r| r.east, true));
        result = result.or(trim(rays.west, |r| r.west, true));
    }
    if diagonal {
        result = result.or(trim(rays.north_east, |r| r.north_east, true));
        result = result.or(trim(rays.north_west, |r| r.north_west, true));
        result = result.or(trim(rays.south_east, |r| r.south_east, false));
        result = result.or(trim(rays.south_west, |r| r.south_west, false));
    }
    // Remove blockers of own color
    let o = occupied.and(own_color_board).not();
    result.and(o)
}

#[inline]
fn rook_moves(board: &Board, square: Square) -> BitBoard {
    sliding_moves(board, square, true, false)
}

#[inline]
fn bishop_moves(board: &Board, square: Square) -> BitBoard {
    sliding_moves(board, square, false, true)
}
#[inline]
fn queen_moves(board: &Board, square: Square) -> BitBoard {
    sliding_moves(board, square, true, true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::{Color, Piece};
    use crate::search::find_best_move;
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
                board.make_move(*white_move);
                println!("{}", board);
            }
        }
    }

    #[test]
    fn best_game() {
        let mut board = Board::empty()
            .set_piece("e1".parse().unwrap(), Piece::King, Color::White)
            .set_piece("a1".parse().unwrap(), Piece::Rook, Color::White)
            .set_piece("e8".parse().unwrap(), Piece::King, Color::Black)
            .set_piece("h8".parse().unwrap(), Piece::Rook, Color::Black);
        println!("\nInitial position:");
        println!("{}", board);
        // Total of 10 ply moves
        for move_num in 1..=20 {
            let best_move =
                find_best_move(&mut board, 2).unwrap_or_else(|| panic!("Found no move"));
            println!(
                "Ply {}: {} plays {} -> {}",
                move_num,
                board.color_to_move(),
                best_move.from().algebraic(),
                best_move.to().algebraic()
            );
            board.make_move(best_move);
            println!("{}", board);
        }
    }
}
