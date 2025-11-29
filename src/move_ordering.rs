use crate::board::{Board, Piece};
use crate::eval::piece_value_mg;
use crate::r#move::Move;

const SCORE_TT_MOVE: i32 = 10_000_000;
const SCORE_WINNING_CAPTURE_BASE: i32 = 8_000_000;
const SCORE_EQUAL_CAPTURE_BASE: i32 = 6_000_000;
const SCORE_KILLER_1: i32 = 4_000_000;
const SCORE_KILLER_2: i32 = 3_000_000;

const MAX_DEPTH: usize = 128;

pub struct KillerMoves {
    killers: [[Option<Move>; 2]; MAX_DEPTH],
}

impl KillerMoves {
    pub fn new() -> Self {
        Self {
            killers: [[None; 2]; MAX_DEPTH],
        }
    }

    pub fn store(&mut self, depth: usize, m: Move) {
        if depth >= MAX_DEPTH {
            return;
        }
        // Shift: killer[1] -> discard, killer[0] -> killer[1], new -> killer[0]
        if Some(m) != self.killers[depth][0] {
            self.killers[depth][1] = self.killers[depth][0];
            self.killers[depth][0] = Some(m);
        }
    }

    #[inline]
    pub fn get(&self, depth: usize) -> [Option<Move>; 2] {
        if depth >= MAX_DEPTH {
            [None, None]
        } else {
            self.killers[depth]
        }
    }

    pub fn clear(&mut self) {
        self.killers = [[None; 2]; MAX_DEPTH];
    }
}

/// Scores a move for ordering. Higher scores = search first.
#[inline]
fn score_move(board: &Board, m: Move, tt_move: Option<Move>, killers: [Option<Move>; 2]) -> i32 {
    // 1. TT move
    if Some(m) == tt_move {
        return SCORE_TT_MOVE;
    }

    // 2. Captures: MVV-LVA
    if m.capture() {
        let victim = board.piece_at_square(m.to());
        let attacker = board.piece_at_square(m.from());

        if let (Some((victim_piece, _)), Some((attacker_piece, _))) = (victim, attacker) {
            let victim_value = piece_value_mg(victim_piece);
            let attacker_value = piece_value_mg(attacker_piece);

            // Winning/equal captures scored higher
            if victim_value >= attacker_value {
                return SCORE_WINNING_CAPTURE_BASE + victim_value - attacker_value;
            } else {
                return SCORE_EQUAL_CAPTURE_BASE + victim_value - attacker_value;
            }
        }
    }

    // 3. Killer moves (quiet moves only)
    if Some(m) == killers[0] {
        return SCORE_KILLER_1;
    }
    if Some(m) == killers[1] {
        return SCORE_KILLER_2;
    }

    // 4. Default (history heuristic could go here)
    0
}

/// Move picker that finds the best move from remaining moves without sorting all.
pub struct MovePicker {
    moves: Vec<Move>,
    scores: Vec<i32>,
    current: usize,
}

impl MovePicker {
    pub fn new(
        board: &Board,
        moves: Vec<Move>,
        tt_move: Option<Move>,
        killers: [Option<Move>; 2],
    ) -> Self {
        let scores: Vec<i32> = moves
            .iter()
            .map(|&m| score_move(board, m, tt_move, killers))
            .collect();

        Self {
            moves,
            scores,
            current: 0,
        }
    }

    /// Pick the next best move. Returns None when exhausted.
    pub fn next(&mut self) -> Option<Move> {
        if self.current >= self.moves.len() {
            return None;
        }

        // Find the index of the best remaining move
        let mut best_idx = self.current;
        let mut best_score = self.scores[self.current];

        for i in (self.current + 1)..self.moves.len() {
            if self.scores[i] > best_score {
                best_score = self.scores[i];
                best_idx = i;
            }
        }

        // Swap to front
        self.moves.swap(self.current, best_idx);
        self.scores.swap(self.current, best_idx);

        let m = self.moves[self.current];
        self.current += 1;
        Some(m)
    }
}
