use crate::board::{Board, Color, Piece};
use crate::r#move::Move;
use crate::precomputed::CastleSide;
use crate::square::Square;
use std::collections::HashMap;
use std::sync::OnceLock;

// Minimal deterministic PRNG to avoid external rand dependency (SplitMix64)
#[derive(Copy, Clone)]
struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    #[inline]
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    #[inline]
    fn next_u64(&mut self) -> u64 {
        // Public domain splitmix64
        let mut z = {
            self.state = self.state.wrapping_add(0x9E3779B97F4A7C15);
            self.state
        };
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum NodeType {
    Exact,
    LowerBound,
    UpperBound,
}

#[derive(Copy, Clone)]
pub struct TTEntry {
    pub key: u64,
    pub depth: u8,
    pub score: i32,
    pub flag: NodeType,
    pub best_move: Option<Move>,
}

pub struct TranspositionTable {
    pub map: HashMap<u64, TTEntry>,
    pub max_entries: usize,
}

impl TranspositionTable {
    pub fn new(max_entries: usize) -> Self {
        Self {
            map: HashMap::with_capacity(max_entries.min(1 << 20)),
            max_entries,
        }
    }

    #[inline]
    pub fn get(&self, key: u64) -> Option<&TTEntry> {
        self.map.get(&key)
    }

    #[inline]
    pub fn store(&mut self, entry: TTEntry) {
        if self.map.len() >= self.max_entries {
            // Very simple aging: clear the table when full.
            self.map.clear();
        }
        // Replace only if deeper or not present
        match self.map.get(&entry.key) {
            Some(old) if old.depth > entry.depth => (),
            _ => {
                self.map.insert(entry.key, entry);
            }
        }
    }
}

// ---------------- Zobrist Hashing ----------------
// Layout: [color][piece][square]
// color: 0 = white, 1 = black
// piece: 0..5 = King, Queen, Rook, Bishop, Knight, Pawn (must match Board::Piece order)
struct Zobrist {
    pieces: [[[u64; 64]; Piece::COUNT]; Color::COUNT],
    side_to_move: u64,
    castling: [[u64; CastleSide::COUNT]; Color::COUNT], // [color][side]
    en_passant_file: [u64; 8],
}

static ZOBRIST: OnceLock<Zobrist> = OnceLock::new();

fn init_zobrist() -> Zobrist {
    // Fixed seed for reproducibility across runs/builds.
    let mut rng = SplitMix64::new(0xC0FFEE_F00D_FACADE);
    let mut pieces = [[[0u64; 64]; 6]; 2];
    for c in 0..2 {
        for p in 0..6 {
            for s in 0..64 {
                pieces[c][p][s] = rng.next_u64();
            }
        }
    }
    let side_to_move = rng.next_u64();
    let mut castling = [[0u64; 2]; 2];
    for c in 0..2 {
        for side in 0..2 {
            castling[c][side] = rng.next_u64();
        }
    }
    let mut en_passant_file = [0u64; 8];
    for f in 0..8 {
        en_passant_file[f] = rng.next_u64();
    }
    Zobrist {
        pieces,
        side_to_move,
        castling,
        en_passant_file,
    }
}

#[inline]
pub fn position_key(b: &Board) -> u64 {
    let z = ZOBRIST.get_or_init(init_zobrist);

    let mut h: u64 = 0;

    // Pieces
    // Board::pieces holds bitboards per piece type, color is in b.white bitboard
    for piece_idx in 0..6 {
        let mut bb = b.pieces[piece_idx].0;
        while bb != 0 {
            let sq = bb.trailing_zeros() as u8;
            bb &= bb - 1;
            let is_white = (b.white.0 >> sq) & 1 == 1;
            let color = if is_white { 0 } else { 1 };
            h ^= z.pieces[color][piece_idx][sq as usize];
        }
    }

    // Side to move
    if !b.white_to_move {
        // Common convention: XOR if black to move
        h ^= z.side_to_move;
    }

    // Castling rights
    for color in 0..2 {
        for side in 0..2 {
            if b.castling_rights[color][side] {
                h ^= z.castling[color][side];
            }
        }
    }

    // En passant file: include only the file if an ep square is set
    if b.en_passant != Square::ILLEGAL_SQUARE {
        let file = (b.en_passant.0 % 8) as usize;
        h ^= z.en_passant_file[file];
    }

    h
}
