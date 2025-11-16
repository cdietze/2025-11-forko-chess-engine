use crate::bitboard_ops::BitBoard;
use crate::board::{Board, Color, Piece};

// PeSTO's Evaluation Function ported to Rust
// Source: https://www.chessprogramming.org/PeSTO%27s_Evaluation_Function

// Notes on indexing/mirroring:
// - In this codebase, squares are indexed as rank * 8 + file with a1 = 0, h8 = 63.
// - PeSTO PSTs are provided for the white side from a1 perspective. For black pieces
//   we mirror the square vertically by flipping the rank using sq ^ 56.

// Midgame and Endgame piece values (PeSTO)
const MG_VALUE: [i32; Piece::COUNT] = [
    0,    // King
    1025, // Queen
    477,  // Rook
    365,  // Bishop
    337,  // Knight
    82,   // Pawn
];
const EG_VALUE: [i32; Piece::COUNT] = [
    0,   // King
    936, // Queen
    512, // Rook
    297, // Bishop
    281, // Knight
    94,  // Pawn
];

// Game phase increments per piece type (PeSTO). Pawns and kings don't count.
const PHASE_INC: [i32; Piece::COUNT] = [0, 4, 2, 1, 1, 0];
const TOTAL_PHASE: i32 = 24; // 2*(4+2+1+1)

#[rustfmt::skip]
const MG_PAWN_PST: [i32; 64] = [
      0,   0,   0,   0,   0,   0,   0,   0,
     98, 134,  61,  95,  68, 126,  34, -11,
     -6,   7,  26,  31,  65,  56,  25, -20,
    -14,  13,   6,  21,  23,  12,  17, -23,
    -27,  -2,  -5,  12,  17,   6,  10, -25,
    -26, -26, -22,  -9,  -1,  -7, -19, -26,
    -35, -11, -31, -12,  -1, -19,  -9, -40,
      0,   0,   0,   0,   0,   0,   0,   0,
];
#[rustfmt::skip]
const EG_PAWN_PST: [i32; 64] = [
      0,   0,   0,   0,   0,   0,   0,   0,
     178, 173, 158, 134, 147, 132, 165, 187,
      94, 100,  85,  67,  56,  53,  82,  84,
      32,  24,  13,   5,  -2,   4,  17,  17,
      13,   9,  -3, - 7, - 7, - 8,   3,  -1,
       4,   7, - 6,   1,  0, - 5, - 1, - 8,
      13,   8,   8,  10,  13,   0,   2, - 7,
      0,   0,   0,   0,   0,   0,   0,   0,
];

#[rustfmt::skip]
const MG_KNIGHT_PST: [i32; 64] = [
    -167, - 89, - 34, - 49,  61, - 97, - 15, -107,
     -73,  -41,  72,  36,  23,  62,   7, - 17,
     -47,   60,  37,  65,  84, 129,  73,  44,
      -9,   17,  19,  53,  37,  69,  18,  22,
     -13,    4,  16,  13,  28,  19,  21,  -8,
     -23,  - 9,  12,  10,  19,  17,  25, - 16,
     -29, - 53, - 12, - 3, - 1,  18, - 14, - 19,
    -105, - 21, - 58, - 33, - 17, - 28, - 19, - 23,
];
#[rustfmt::skip]
const EG_KNIGHT_PST: [i32; 64] = [
    -58, - 38, - 13, - 28, - 31, - 27, - 63, - 99,
    - 25,  - 8, - 25,  -2,  -9, - 25, - 24, - 52,
    - 24, - 20,  10,   9,  -1,  -9, - 19, - 41,
     -17,   3,  22,  22,  22,  11,   8, - 18,
     -18,  - 6,  16,  25,  16,  17,   4, - 18,
     -23,  - 3,  -1,  15,  10,  -3,  - 20, - 22,
     -42, - 20, - 10,  -5, - 2, - 20, - 23, - 44,
     -29, - 51, - 23, - 15, - 22, - 18, - 50, - 64,
];

#[rustfmt::skip]
const MG_BISHOP_PST: [i32; 64] = [
    -29,   4, - 82, - 37, - 25, - 42,   7, - 8,
    - 26,  16, - 18, - 13,  30,  59,  18, - 47,
    - 16,  37,  43,  40,  35,  50,  37,  -2,
     -4,   5,  19,  50,  37,  37,   7,  -2,
     -6,  13,  13,  26,  34,  12,  10,   4,
      0,  15,  15,  15,  14,  27,  18,  10,
      4,  15,  16,   0,   7,  21,  33,   1,
    -33,  - 3, - 14, - 21, - 13, - 12, - 39, - 21,
];
#[rustfmt::skip]
const EG_BISHOP_PST: [i32; 64] = [
    -14, - 21, - 11, - 8, - 7, - 9, - 17, - 24,
    - 8,   - 4,   7, - 12, - 3, - 13, - 4, - 14,
     2,    - 8,   0,  -1,  -2,   6,   0,   4,
    - 3,     9,  12,   9,  14,  10,   3,   2,
    - 6,     3,  13,  19,   7,  10,  -3,  - 9,
    -12,   - 3,   8,  10,  13,   3,  -7, - 15,
    -14,   -18, - 7,  -1,   4,  -9, - 15, - 27,
    -23,   - 9, - 23,  -5,  -9, - 16,  -5, - 17,
];

#[rustfmt::skip]
const MG_ROOK_PST: [i32; 64] = [
     32,  42,  32,  51,  63,   9,  31,  43,
     27,  32,  58,  62,  80,  67,  26,  44,
     -5,  19,  26,  36,  17,  45,  61,  16,
    -24, -11,   7,  26,  24,  35,  -8, -20,
    -36, -26, -12,  -1,   9,  -7,   6, -23,
    -45, -25, -16, -17,   3,   0, - 5, -33,
    -44, -16, -20,  -9,  -1,  11, - 6, -71,
     -19, -13,   1,  17,  16,   7, -37, -26,
];
#[rustfmt::skip]
const EG_ROOK_PST: [i32; 64] = [
     13,  10,  18,  15,  12,  12,   8,   5,
     11,  13,  13,  11,  -3,   3,   8,   3,
      7,   7,   7,   5,   4,  -3,  -5,  -3,
      4,   3,  13,   1,   2,   1,  -1,   2,
      3,   5,   8,   4,  -5,  -6,  -8, -11,
     -4,   0,  -5,  -1,  -7, -12,  -8,  -16,
     -6,  -6,   0,   2,  -9,  -9,  -11,  -3,
      0,   -6,   7,   6,   6,   4,   0,  -15,
];

#[rustfmt::skip]
const MG_QUEEN_PST: [i32; 64] = [
    -28,   0,  29,  12,  59,  44,  43,  45,
    - 24, -39, - 5,   1, -16,  57,  28,  54,
    -13, -17,   7,   8,  29,  56,  47,  57,
    -27, -27, -16, -16, - 1,  17, - 2,   1,
     -9, -26, - 9, -10, - 2, - 4,   3,  -3,
    -14,   2, -11,  -2,  -5,   2,  14,   5,
    -35, - 8,  11,   2,   8,  15,  -3,   1,
     -1, -18, - 9,  10, -15, -25, -31, -50,
];
#[rustfmt::skip]
const EG_QUEEN_PST: [i32; 64] = [
      -9,  22,  22,  27,  27,  19,  10,  20,
     -17,  20,  32,  41,  58,  25,  30,   0,
     -20,   6,   9,  49,  47,  35,  19,   9,
       3,  22,  24,  45,  57,  40,  57,  36,
     -18,  28,  19,  47,  31,  34,  39,  23,
     -16, - 27,  15,   6,   9,  17,  10,   5,
     -22, - 23, - 30, - 16, - 16, - 23, - 36, - 32,
     -33, - 28, - 22, - 43,  -5, - 32, - 20, - 41,
];

#[rustfmt::skip]
const MG_KING_PST: [i32; 64] = [
    -65,  23,  16, -15, -56, -34,   2,  13,
     29,  -1, -20, - 7, - 8, - 4, - 38, -29,
     -9,  24,   2, -16, -20,   6,  22, -22,
    -17,  - 20, - 12, -27, -30, -25, -14, -36,
    -49, - 1, - 27, -39, -46, -44, -33, -51,
    -14, -14, - 22, -46, -44, -30, -15, -27,
      1,   7, -  8, -64, -43, -16,   9,   8,
    -15,  36,  12, -54,   8, -28,  24,  14,
];
#[rustfmt::skip]
const EG_KING_PST: [i32; 64] = [
    -74, -35, -18, -18, -11,  15,   4, -17,
    -12,  17,  14,  17,  17,  38,  23,  11,
     10,  17,  23,  15,  20,  45,  44,  13,
     -8,  22,  24,  27,  26,  33,  26,   3,
    -18,  -4,  21,  24,  27,  23,   9, -11,
    -19,  -3,  11,  21,  23,  16,   7,  -9,
    -27, -11,   4,  13,  14,   4,  -5, -17,
    -53, -34, -21, -11, -28, -14, -24, -43,
];

#[inline]
const fn mirror_sq_for_black(sq: u8) -> u8 {
    sq ^ 56 // flip ranks (vertical mirror)
}

#[inline]
fn pst_for(piece: Piece, mg: bool) -> &'static [i32; 64] {
    match (piece, mg) {
        (Piece::Pawn, true) => &MG_PAWN_PST,
        (Piece::Pawn, false) => &EG_PAWN_PST,
        (Piece::Knight, true) => &MG_KNIGHT_PST,
        (Piece::Knight, false) => &EG_KNIGHT_PST,
        (Piece::Bishop, true) => &MG_BISHOP_PST,
        (Piece::Bishop, false) => &EG_BISHOP_PST,
        (Piece::Rook, true) => &MG_ROOK_PST,
        (Piece::Rook, false) => &EG_ROOK_PST,
        (Piece::Queen, true) => &MG_QUEEN_PST,
        (Piece::Queen, false) => &EG_QUEEN_PST,
        (Piece::King, true) => &MG_KING_PST,
        (Piece::King, false) => &EG_KING_PST,
    }
}

#[inline]
fn piece_value(piece: Piece, mg: bool) -> i32 {
    let idx = piece.idx();
    if mg { MG_VALUE[idx] } else { EG_VALUE[idx] }
}

#[inline]
fn eval_side(board: &Board, color: Color, mg: bool) -> i32 {
    let mut score = 0;
    for &p in &Piece::ALL {
        let mut bb = board.pieces(p, color);
        let pst = pst_for(p, mg);
        while bb.is_not_empty() {
            let sq = bb.bit_scan_forward();
            // pop lsb
            bb = BitBoard(bb.0 & (bb.0 - 1));
            let idx = match color {
                Color::White => sq,
                Color::Black => mirror_sq_for_black(sq),
            } as usize;
            score += piece_value(p, mg) + pst[idx];
        }
    }
    score
}

#[inline]
fn game_phase(board: &Board) -> i32 {
    let mut phase = 0;
    for &p in &Piece::ALL {
        let inc = PHASE_INC[p.idx()];
        if inc == 0 {
            continue;
        }
        for &c in &[Color::White, Color::Black] {
            let mut bb = board.pieces(p, c);
            while bb.is_not_empty() {
                phase += inc;
                bb = BitBoard(bb.0 & (bb.0 - 1));
            }
        }
    }
    if phase > TOTAL_PHASE {
        TOTAL_PHASE
    } else {
        phase
    }
}

pub fn eval(board: &Board) -> i32 {
    // Tapered evaluation: blend midgame and endgame based on material phase
    let mg_w = eval_side(board, Color::White, true);
    let mg_b = eval_side(board, Color::Black, true);
    let eg_w = eval_side(board, Color::White, false);
    let eg_b = eval_side(board, Color::Black, false);

    let mg = mg_w - mg_b;
    let eg = eg_w - eg_b;

    let phase = game_phase(board);
    // scale scores to side-to-move perspective (tempo bonus included below)
    let blended = (mg * phase + eg * (TOTAL_PHASE - phase)) / TOTAL_PHASE.max(1);

    // Tempo bonus as in PeSTO
    let tempo = 10;
    let stm_bonus = if board.white_to_move { tempo } else { -tempo };
    blended + stm_bonus
}
