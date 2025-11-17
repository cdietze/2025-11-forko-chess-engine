use crate::bitboard::BitBoard;
use crate::board::{Board, Color, Piece};
use crate::geometry::{Dir8, between_bb, line_bb};
use crate::r#move::Move;
use crate::precomputed::{king_moves, ray_attacks};
use crate::square::Square;

/// Generates a list of *legal* moves from given board.
pub fn generate_moves(board: &Board) -> Vec<Move> {
    let mut v = Vec::new();

    let own_pieces: BitBoard = if board.white_to_move {
        board.white
    } else {
        board.white.not()
    };
    let opp_pieces: BitBoard = own_pieces.not();
    let occupied = board.occupied();
    let own_king_bb = board.kings().and(own_pieces);
    let own_king = Square(own_king_bb.bit_scan_forward());
    assert!(own_king.is_valid());

    let opp_rq = board.pieces[Piece::Rook.idx()]
        .or(board.pieces[Piece::Queen.idx()])
        .and(opp_pieces);

    let pinned = pinned(own_king, occupied, own_pieces, opp_rq);
    let king_attack_map = king_attack_map(board, board.color_to_move().opposite());
    let attacks_to_king = attacks_to_king(board);
    let num_checks = attacks_to_king.0.count_ones();
    if num_checks == 0 {
        let not_own_pieces_bb = occupied.and(board.own_color_board()).not();
        add_king_moves(
            AddKingMovesProps {
                king_square: own_king,
                to_mask: not_own_pieces_bb,
                king_attack_map,
            },
            &mut v,
        );
        add_rook_moves(
            AddRookMovesProps {
                rooks: board.pieces(Piece::Rook, board.color_to_move()),
                king_square: own_king,
                occupied,
                pinned,
                to_mask: not_own_pieces_bb,
            },
            &mut v,
        );
    } else if num_checks == 1 {
        let not_own_pieces_bb = occupied.and(board.own_color_board()).not();
        let checking_piece = attacks_to_king.bit_scan_forward();
        let attack_line_bb = between_bb(own_king, Square(checking_piece));
        // King moves to safe square
        add_king_moves(
            AddKingMovesProps {
                king_square: own_king,
                to_mask: not_own_pieces_bb,
                king_attack_map,
            },
            &mut v,
        );
        // Capture the checking piece or block the checking piece
        add_rook_moves(
            AddRookMovesProps {
                rooks: board.pieces(Piece::Rook, board.color_to_move()),
                king_square: own_king,
                occupied,
                pinned,
                to_mask: attacks_to_king.or(attack_line_bb),
            },
            &mut v,
        )
    } else if num_checks > 1 {
        let not_own_pieces_bb = occupied.and(board.own_color_board()).not();
        // In double check, only king moves to safe squares are possible
        add_king_moves(
            AddKingMovesProps {
                king_square: own_king,
                to_mask: not_own_pieces_bb,
                king_attack_map,
            },
            &mut v,
        );
    }
    v
}

struct AddKingMovesProps {
    king_square: Square,
    to_mask: BitBoard,
    king_attack_map: BitBoard,
}

fn add_king_moves(props: AddKingMovesProps, v: &mut Vec<Move>) {
    let tos = king_moves(props.king_square);
    // Don't capture own pieces
    let tos = tos.and(props.to_mask);
    // Don't move into check
    let tos = tos.and(props.king_attack_map.not());
    tos.for_each_set_bit(|to_square| {
        v.push(Move::new(props.king_square, to_square));
        true
    });
}

struct AddRookMovesProps {
    rooks: BitBoard,
    king_square: Square,
    occupied: BitBoard,
    pinned: BitBoard,
    to_mask: BitBoard,
}
fn add_rook_moves(props: AddRookMovesProps, v: &mut Vec<Move>) {
    let occupied = props.occupied;
    props.rooks.for_each_set_bit(|rook_square| {
        let mut tos = rook_attacks(rook_square, occupied);
        tos = tos.and(props.to_mask);
        if props.pinned.has_square(rook_square) {
            tos = tos.and(line_bb(props.king_square, rook_square))
        }
        tos.for_each_set_bit(|to_square| {
            v.push(Move::new(rook_square, to_square));
            true
        });
        true
    });
}

fn rook_attacks(rook_square: Square, occ: BitBoard) -> BitBoard {
    file_attacks(rook_square, occ).or(rank_attacks(rook_square, occ))
}

fn file_attacks(square: Square, occ: BitBoard) -> BitBoard {
    postive_ray_attacks(occ, Dir8::East, square).or(negative_ray_attacks(occ, Dir8::West, square))
}

fn rank_attacks(square: Square, occ: BitBoard) -> BitBoard {
    postive_ray_attacks(occ, Dir8::North, square).or(negative_ray_attacks(occ, Dir8::South, square))
}

fn xray_rook(rook_square: Square, occ: BitBoard, blockers: BitBoard) -> BitBoard {
    let attacks = rook_attacks(rook_square, occ);
    let blockers = blockers.and(attacks);
    attacks.xor(rook_attacks(rook_square, occ.xor(blockers)))
}

/// Returns a `BitBoard` containing all squares with pinned pieces.
fn pinned(king_square: Square, occ: BitBoard, own_pieces: BitBoard, opp_rq: BitBoard) -> BitBoard {
    let mut pinned = BitBoard::EMPTY;
    let pinners = xray_rook(king_square, occ, own_pieces).and(opp_rq);
    pinners.for_each_set_bit(|square| {
        let p = between_bb(king_square, square).and(own_pieces);
        pinned = pinned.or(p);
        true
    });
    pinned
}

/// Returns a `BitBoard` containing all squares where the king would be attacked.
pub fn king_attack_map(board: &Board, opposing_color: Color) -> BitBoard {
    let mut map = BitBoard::EMPTY;
    // remove own king
    let occupied = board
        .occupied()
        .and(board.kings().and(board.own_color_board()).not());
    board
        .pieces(Piece::King, opposing_color)
        .for_each_set_bit(|king_square| {
            map = map.or(king_moves(king_square));
            true
        });
    board
        .pieces(Piece::Rook, opposing_color)
        .for_each_set_bit(|rook_square| {
            map = map.or(rook_attacks(rook_square, occupied));
            true
        });
    map
}

/// Returns a `BitBoard` containing all pieces currently attacking the king.
pub fn attacks_to_king(board: &Board) -> BitBoard {
    let king = board
        .kings()
        .and(board.own_color_board())
        .bit_scan_forward();
    let occupied = board.occupied();
    let rook_attackers = rook_attacks(Square(king), occupied)
        .and(board.pieces(Piece::Rook, board.color_to_move().opposite()));
    rook_attackers
}

fn postive_ray_attacks(occ: BitBoard, ray: Dir8, square: Square) -> BitBoard {
    let attacks = ray_attacks(square, ray);
    let blocker = occ.and(attacks);
    if blocker.is_not_empty() {
        let b = blocker.bit_scan_forward();
        return attacks.xor(ray_attacks(Square(b), ray));
    }
    attacks
}

fn negative_ray_attacks(occ: BitBoard, ray: Dir8, square: Square) -> BitBoard {
    let attacks = ray_attacks(square, ray);
    let blocker = occ.and(attacks);
    if blocker.is_not_empty() {
        let b = blocker.bit_scan_backward();
        return attacks.xor(ray_attacks(Square(b), ray));
    }
    attacks
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::{Color, Piece};
    use crate::search::find_best_move;
    use crate::util::assert_eq_unordered;
    use rand::prelude::IndexedRandom;
    use std::collections::BTreeSet;

    fn move_list(list: &[&str]) -> Vec<Move> {
        list.iter()
            .copied()
            .map(|m| m.parse::<Move>().unwrap())
            .collect()
    }
    #[test]
    fn xray_rook_should_be_correct() {
        let blockers = BitBoard::try_from_coords(["a3", "a6"]).unwrap();
        let xray = xray_rook(Square(0), blockers, blockers);
        assert_eq!(xray, BitBoard::try_from_coords(["a4", "a5", "a6"]).unwrap());
    }

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
        let board = Board::from_fen("5k1K/6r1/8/8/8/8/8/R7 w - - 0 1");
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
    fn black_rook_should_move_correctly_from_e4() {
        let board = Board::from_fen("5K1k/6R1/8/8/4r3/8/8/8 b - - 0 1");
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
        let board = Board::from_fen("8/8/8/8/R4r2/8/1r6/K1k5 w - - 0 1");
        let moves = generate_moves(&board);
        assert_move_sources(&moves, &["a4"]);
        assert_move_destinations(
            &moves,
            &[
                "a2", "a3", "a5", "a6", "a7", "a8", "b4", "c4", "d4", "e4", "f4",
            ],
        );
    }

    #[test]
    fn should_find_no_moves_when_checkmate() {
        let mut board = Board::empty()
            .set_piece("e6".parse().unwrap(), Piece::King, Color::White)
            .set_piece("a8".parse().unwrap(), Piece::Rook, Color::White)
            .set_piece("e8".parse().unwrap(), Piece::King, Color::Black)
            .set_color_to_move(Color::Black);
        let moves = generate_moves(&board);
        assert_eq!(moves.len(), 0);
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
    fn should_not_move_pinned_rook_and_leave_king_in_check() {
        let board = Board::from_fen("8/8/8/8/8/8/5kr1/5rRK w - - 0 1");
        let moves = generate_moves(&board);
        let expected = move_list(&["g1f1"]);
        assert_eq!(moves, expected, "unexpected moves: {:?}", moves);
    }

    #[test]
    fn when_in_check_should_evade() {
        let board = Board::from_fen("7k/8/8/8/1R6/8/8/Kr6 w - - 0 1");
        let moves = generate_moves(&board);
        let expected = move_list(&["a1a2", "a1b1", "b4b1"]);
        assert_eq_unordered(&moves, &expected);
    }
    #[test]
    fn when_in_check_should_block() {
        let board = Board::from_fen("7k/8/8/8/8/8/1R6/K1r5 w - - 0 1");
        let moves = generate_moves(&board);
        let expected = move_list(&["a1a2", "b2b1"]);
        assert_eq_unordered(&moves, &expected);
    }
    #[test]
    fn should_solve_king_check_position() {
        let board = Board::from_fen("7k/8/8/8/1RR5/8/8/K1r3R1 w - - 0 1");
        let moves = generate_moves(&board);
        let expected = move_list(&["a1a2", "a1b2", "b4b1", "c4c1", "g1c1"]);
        assert_eq_unordered(&moves, &expected);
    }

    #[test]
    fn should_solve_double_check_position() {
        let board = Board::from_fen("7k/3r2R1/8/8/5R2/8/8/3K2r1 w - - 0 1");
        let moves = generate_moves(&board);
        let expected = move_list(&["d1c2", "d1e2"]);
        assert_eq_unordered(&moves, &expected);
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
            let best_move = find_best_move(&mut board, 2)
                .move_
                .unwrap_or_else(|| panic!("Found no move"));
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
