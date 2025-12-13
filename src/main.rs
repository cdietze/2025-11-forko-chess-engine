mod bitboard;
mod bitboard_ops;
mod board;
mod eval;
mod fen;
mod geometry;
mod r#move;
mod move_gen;
mod move_ordering;
mod perft;
mod precomputed;
mod search;
mod square;
mod transposition;
mod uci;
mod util;
mod wasm;
mod zobrist;

fn main() {
    // Delegate to the UCI loop implemented in a separate, extensible module.
    uci::run();
}
