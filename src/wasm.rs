use std::sync::{Mutex, OnceLock};
use wasm_bindgen::prelude::*;

use crate::board::Board;
use crate::search::find_best_move;
use crate::transposition::TranspositionTable;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

struct EngineState {
    board: Board,
    tt: TranspositionTable,
}

fn state() -> &'static Mutex<EngineState> {
    static STATE: OnceLock<Mutex<EngineState>> = OnceLock::new();
    STATE.get_or_init(|| {
        Mutex::new(EngineState {
            board: Board::default(),
            tt: TranspositionTable::new(1_000_000),
        })
    })
}

#[wasm_bindgen]
pub fn version() -> String {
    crate::util::engine_display_name()
}

#[wasm_bindgen]
pub fn set_fen(fen: &str) {
    let mut guard = state().lock().expect("engine state mutex poisoned");
    guard.board = Board::from_fen(fen);
}

#[wasm_bindgen]
pub fn best_move() -> String {
    let mut guard = state().lock().expect("engine state mutex poisoned");
    // Work on a copy of the board to satisfy the borrow checker (Board is Copy)
    let mut board_copy = guard.board;
    // Choose a modest default depth suitable for WASM execution
    let depth: u8 = 6;
    let result = find_best_move(&mut board_copy, depth, &mut guard.tt);
    match result.move_ {
        Some(mv) => mv.to_string(),
        None => String::new(),
    }
}
