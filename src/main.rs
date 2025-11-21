use crate::board::Board;
use crate::util::with_separator;

mod bitboard;
mod bitboard_ops;
mod board;
mod eval;
mod fen;
mod geometry;
mod r#move;
mod move_gen;
mod perft;
mod precomputed;
mod search;
mod square;
mod util;

fn main() {
    let mut board = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");

    println!(
        "Starting position ({} to move):\n{}",
        board.color_to_move(),
        board
    );

    // Play up to 20 plies from the given position using the search
    for ply in 1..=100 {
        let result = search::find_best_move(&mut board, 4);
        match result.move_ {
            Some(m) => {
                println!(
                    "Ply {ply}: best move = {} (score: {})",
                    m,
                    with_separator(result.score)
                );
                board.make_move(m);
                println!(
                    "After ply {ply} ({} to move):\n{}\n{}",
                    board.color_to_move(),
                    board.to_fen(),
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
