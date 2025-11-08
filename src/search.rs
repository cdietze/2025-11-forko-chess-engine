use crate::board::Piece::{Bishop, Knight, Pawn, Queen, Rook};
use crate::board::{Board, Piece};
use crate::r#move::Move;
use crate::move_gen::generate_moves;

#[derive(Debug)]
struct SearchInfo {
    node_count: u64,
}
fn nega_max(board: &mut Board, depth: u8, info: &mut SearchInfo) -> i32 {
    info.node_count += 1;
    if depth == 0 {
        return eval(board);
    }
    let mut max = i32::MIN;
    let moves = generate_moves(board);
    for m in moves {
        let mut b = board.clone();
        b.make_move(m);
        let score = -nega_max(&mut b, depth - 1, info);
        if (score > max) {
            max = score;
        }
    }
    max
}

pub fn find_best_move(board: &mut Board, depth: u8) -> Option<Move> {
    let mut info = SearchInfo { node_count: 0 };
    let mut max = i32::MIN;
    let mut best_move = None;
    let moves = generate_moves(board);
    for m in moves {
        let mut b = board.clone();
        b.make_move(m);
        let score = -nega_max(&mut b, depth - 1, &mut info);
        if (score > max) {
            max = score;
            best_move = Some(m);
        }
    }
    println!("find_best_move done: {:?}", info);
    best_move
}

/// Evaluation relative to side to move.
fn eval(board: &Board) -> i32 {
    let piece_diff = |p: Piece| -> i32 {
        let p = board.pieces[p.idx()];
        p.and(board.white).0.count_ones() as i32 - p.and(board.white.not()).0.count_ones() as i32
    };
    let mut score = piece_diff(Pawn) * 100;
    score += piece_diff(Knight) * 300;
    score += piece_diff(Bishop) * 350;
    score += piece_diff(Rook) * 500;
    score += piece_diff(Queen) * 900;
    score * if (board.white_to_move) { 1 } else { -1 }
}
