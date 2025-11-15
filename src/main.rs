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
mod util;

fn main() {
    // Example position: simple rook-and-king vs king
    // let mut board = Board::empty()
    //     .set_piece("b6".parse().unwrap(), Piece::King, Color::White)
    //     .set_piece("b1".parse().unwrap(), Piece::Rook, Color::White)
    //     .set_piece("a8".parse().unwrap(), Piece::King, Color::Black)
    //     .set_color_to_move(Color::White);
    let mut board = Board::from_fen("8/8/8/3k4/8/R7/R7/K7 w - - 0 1");

    println!(
        "Starting position ({} to move):\n{}",
        board.color_to_move(),
        board
    );

    // Play up to 20 plies from the given position using the search
    for ply in 1..=20 {
        let result = search::find_best_move(&mut board, 6);
        match result.move_ {
            Some(m) => {
                println!("Ply {ply}: best move = {} (score: {})", m, result.score);
                board.make_move(m);
                println!(
                    "After ply {ply} ({} to move):\n{}",
                    board.color_to_move(),
                    board
                );
            }
            None => {
                println!("Ply {ply}: no legal move found. Score: {}", result.score);
                println!(
                    "Final position ({} to move):\n{}",
                    board.color_to_move(),
                    board
                );
                break;
            }
        }
    }
}
