use crate::bitboard::BitBoard;
use crate::board::{Board, Color, Piece};
use crate::r#move::Move;
use crate::precomputed::{Dir8, KING_MOVES, RAYS};
use crate::square::Square;

enum Dir4 {
    Rank,
    File,
    Diagonal,
    AntiDiagonal,
}
impl Dir4 {
    pub const COUNT: usize = 4;

    pub const ALL: [Dir4; Dir4::COUNT] =
        [Dir4::Rank, Dir4::File, Dir4::Diagonal, Dir4::AntiDiagonal];
    #[inline]
    pub const fn idx(self) -> usize {
        self as usize
    }
}

impl BitBoard {}

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
    // println!("#generate_moves, occupied:\n{:?}", occupied);
    // println!("#generate_moves, own_pieces:\n{:?}", own_pieces);
    let own_king = Square(board.kings().and(own_pieces).bit_scan_forward());
    assert!(own_king.is_valid());

    let opp_rq = board.pieces[Piece::Rook.idx()]
        .or(board.pieces[Piece::Queen.idx()])
        .and(opp_pieces);

    let pinned = pinned(own_king, occupied, own_pieces, opp_rq);
    // println!("pinned:\n{:?}", pinned);
    let king_attack_map = generate_king_attack_map(board, board.color_to_move().opposite());
    let not_own_pieces_bb = occupied.and(own_pieces).not();

    board
        .pieces(Piece::King, board.color_to_move())
        .for_each_set_bit(|king_square| {
            let tos = KING_MOVES[king_square.0 as usize];
            // Don't capture own pieces
            let tos = tos.and(occupied.and(own_pieces).not());
            // Don't move into check
            let tos = tos.and(king_attack_map.not());
            tos.for_each_set_bit(|to_square| {
                v.push(Move::new(king_square, to_square));
                true
            })
        });
    gen_rook_moves(board, own_king, pinned, &mut v);
    v
}

fn gen_rook_moves(board: &Board, own_king: Square, pinned: BitBoard, v: &mut Vec<Move>) {
    let occupied = board.occupied();
    let not_own_pieces_bb = occupied.and(board.own_color_board()).not();
    board
        .pieces(Piece::Rook, board.color_to_move())
        .for_each_set_bit(|rook_square| {
            let mut tos = rook_attacks(rook_square, occupied);
            tos = tos.and(not_own_pieces_bb);
            if pinned.has_square(rook_square) {
                tos = tos.and(line_bb(own_king, rook_square))
            }
            println!(
                "#gen_rook_moves, rook_square {:?}, tos:\n{:?}",
                rook_square, tos
            );
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

/// Calculates the set of squares that lie between two given chessboard squares as a `BitBoard`
/// (exclusive of the destination square), considering alignment along files, ranks, or diagonals.
///
/// If the two squares are not aligned (i.e., they do not lie on the same file, rank, or diagonal),
/// an empty `BitBoard` is returned, as there are no obstructed squares along the line between the
/// two points.
/// TODO: precompute in 2d array
fn between_bb(from: Square, to: Square) -> BitBoard {
    // Use same alignment logic as lineBB to keep things DRY
    if from.0 == to.0 {
        return BitBoard::EMPTY;
    }
    let dir = get_dir(from, to);
    let Some(dir) = dir else {
        return BitBoard::EMPTY;
    };

    let (f1, r1) = (from.file() as i8, from.rank() as i8);
    let (f2, r2) = (to.file() as i8, to.rank() as i8);

    // Step direction must point from 'from' towards 'to'
    let (df, dr) = match dir {
        Dir4::File => (0, (r2 - r1).signum()),
        Dir4::Rank => ((f2 - f1).signum(), 0),
        Dir4::Diagonal | Dir4::AntiDiagonal => ((f2 - f1).signum(), (r2 - r1).signum()),
    };

    let mut bb = BitBoard::EMPTY;
    // Start from first square after 'from'
    let mut f = f1 + df;
    let mut r = r1 + dr;
    while f != f2 || r != r2 {
        bb = bb.set_bit(Square::from_file_rank(f as u8, r as u8).0);
        f += df;
        r += dr;
    }
    bb
}

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

/// Returns a `BitBoard` containing all squares that lie on a complete line of two given squares.
/// When from and two are not on a same line, rank, diagonal or anti diagonal, an empty bitboard is returned.
/// Otherwise the result contains all 8 squares of the line, rank, diagonal or anti diagonal on which
/// `from` and `to` lie.
/// TODO: precompute in 2d array
fn line_bb(from: Square, to: Square) -> BitBoard {
    // Determine alignment type first to avoid branching duplication
    let dir = get_dir(from, to);
    let Some(dir) = dir else {
        return BitBoard::EMPTY;
    };

    // For a full line, we backtrack to the edge in the negative direction,
    // then traverse to the opposite edge collecting squares.
    let (mut f, mut r) = (from.file() as i8, from.rank() as i8);
    let (df, dr) = match dir {
        Dir4::File => (0, 1),
        Dir4::Rank => (1, 0),
        Dir4::Diagonal => (1, 1),
        Dir4::AntiDiagonal => (1, -1),
    };

    // Step backwards to the board edge
    while (f - df) >= 0 && (f - df) < 8 && (r - dr) >= 0 && (r - dr) < 8 {
        f -= df;
        r -= dr;
    }

    // Walk forward to the opposite edge collecting all squares
    let mut bb = BitBoard::EMPTY;
    while (0..8).contains(&f) && (0..8).contains(&r) {
        bb = bb.set_bit(Square::from_file_rank(f as u8, r as u8).0);
        f += df;
        r += dr;
    }
    bb
}

fn get_dir(from: Square, to: Square) -> Option<Dir4> {
    let f1 = from.file();
    let r1 = from.rank();
    let f2 = to.file();
    let r2 = to.rank();

    if f1 == f2 {
        return Some(Dir4::File);
    }
    if r1 == r2 {
        return Some(Dir4::Rank);
    }
    let fd = f1 as i8 - f2 as i8;
    let rd = r1 as i8 - r2 as i8;
    if fd.abs() == rd.abs() {
        // Diagonal or anti-diagonal
        return if (f1 as i8 - r1 as i8) == (f2 as i8 - r2 as i8) {
            Some(Dir4::Diagonal)
        } else {
            Some(Dir4::AntiDiagonal)
        };
    }
    None
}

pub fn generate_king_attack_map(board: &Board, opposing_color: Color) -> BitBoard {
    let mut map = BitBoard::EMPTY;
    // remove own king
    let occupied = board
        .occupied()
        .and(board.kings().and(board.own_color_board()).not());
    board
        .pieces(Piece::King, opposing_color)
        .for_each_set_bit(|king_square| {
            map = map.or(KING_MOVES[king_square.0 as usize]);
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

fn postive_ray_attacks(occ: BitBoard, ray: Dir8, square: Square) -> BitBoard {
    let attacks = RAYS[square.0 as usize][ray as usize];
    let blocker = occ.and(attacks);
    if blocker.is_not_empty() {
        let b = blocker.bit_scan_forward();
        return attacks.xor(RAYS[b as usize][ray as usize]);
    }
    attacks
}

fn negative_ray_attacks(occ: BitBoard, ray: Dir8, square: Square) -> BitBoard {
    let attacks = RAYS[square.0 as usize][ray as usize];
    let blocker = occ.and(attacks);
    if blocker.is_not_empty() {
        let b = blocker.bit_scan_backward();
        return attacks.xor(RAYS[b as usize][ray as usize]);
    }
    attacks
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::{Color, Piece};
    use crate::search::find_best_move;
    use rand::prelude::IndexedRandom;
    use std::collections::BTreeSet;
    use std::str::FromStr;

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
        let expected = vec![Move::from_str("g1f1").unwrap()];
        assert_eq!(moves, expected, "unexpected moves: {:?}", moves);
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
