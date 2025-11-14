use crate::board::{Board, Color, Piece};

mod bitboard;
mod bitboard_ops;
mod board;
mod fen;
mod r#move;
mod move_gen;
mod precomputed;
mod search;
mod square;

fn main() {
    println!("Hello, world!");

    let mut board = Board::empty()
        .set_piece("b6".parse().unwrap(), Piece::King, Color::White)
        .set_piece("b1".parse().unwrap(), Piece::Rook, Color::White)
        .set_piece("a8".parse().unwrap(), Piece::King, Color::Black);
    let result = search::find_best_move(&mut board, 4);
    println!("Best move: {:?}", result.move_);
}
