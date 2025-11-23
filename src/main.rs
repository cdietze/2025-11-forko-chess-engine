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
mod uci;
mod util;

fn main() {
    // Delegate to the UCI loop implemented in a separate, extensible module.
    uci::run();
}
