use crate::bitboard::BitBoard;
use crate::board::Color;
use crate::geometry::{Dir4, Dir8, get_dir};
use crate::square::Square;

#[inline]
pub const fn ray_attacks(square: Square, dir: Dir8) -> BitBoard {
    RAYS[square.0 as usize][dir.idx()]
}

#[inline]
pub const fn king_moves(square: Square) -> BitBoard {
    KING_MOVES[square.0 as usize]
}

#[inline]
pub const fn knight_attacks(square: Square) -> BitBoard {
    KNIGHT_ATTACKS[square.0 as usize]
}

/// Returns a `BitBoard` containing all squares that lie on a complete line of two given squares.
/// When from and two are not on a same line, rank, diagonal or anti diagonal, an empty bitboard is returned.
/// Otherwise the result contains all 8 squares of the line, rank, diagonal or anti diagonal on which
/// `from` and `to` lie.
#[inline]
pub fn line_bb(from: Square, to: Square) -> BitBoard {
    LINE_BB[from.0 as usize][to.0 as usize]
}

/// Returns a `BitBoard` containing all squares strictly between `from` and `to` (exclusive),
/// if they are aligned on a rank, file or diagonal; otherwise returns empty.
#[inline]
pub fn between_bb(from: Square, to: Square) -> BitBoard {
    BETWEEN_BB[from.0 as usize][to.0 as usize]
}

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum CastleSide {
    KingSide = 0,
    QueenSide = 1,
}

impl CastleSide {
    pub const COUNT: usize = 2;
    pub const ALL: [Self; Self::COUNT] = [Self::KingSide, Self::QueenSide];

    #[inline]
    pub const fn idx(self) -> usize {
        self as usize
    }
}

pub struct CastlingSetup {
    pub castle_side: CastleSide,
    pub king_from: Square,
    pub king_to: Square,
    pub rook_from: Square,
    pub rook_to: Square,
    pub safe_squares: BitBoard,
    pub empty_squares: BitBoard,
}

static LINE_BB: [[BitBoard; 64]; 64] = {
    let mut table = [[BitBoard(0); 64]; 64];
    let mut i = 0;
    while i < 64 {
        let mut j = 0;
        while j < 64 {
            table[i][j] = compute_line_bb(Square(i as u8), Square(j as u8));
            j += 1;
        }
        i += 1;
    }
    table
};

static BETWEEN_BB: [[BitBoard; 64]; 64] = {
    let mut table = [[BitBoard(0); 64]; 64];
    let mut i = 0;
    while i < 64 {
        let mut j = 0;
        while j < 64 {
            table[i][j] = compute_between_bb(Square(i as u8), Square(j as u8));
            j += 1;
        }
        i += 1;
    }
    table
};

pub const CASTLING_SETUPS: [[CastlingSetup; CastleSide::COUNT]; Color::COUNT] = [
    [
        // White O-O
        CastlingSetup {
            castle_side: CastleSide::KingSide,
            king_from: Square::E1,
            king_to: Square::G1,
            rook_from: Square::H1,
            rook_to: Square::F1,
            safe_squares: BitBoard::from_squares(&[Square::E1, Square::F1, Square::G1]),
            empty_squares: BitBoard::from_squares(&[Square::F1, Square::G1]),
        },
        // White O-O-O
        CastlingSetup {
            castle_side: CastleSide::QueenSide,
            king_from: Square::E1,
            king_to: Square::C1,
            rook_from: Square::A1,
            rook_to: Square::D1,
            safe_squares: BitBoard::from_squares(&[Square::E1, Square::D1, Square::C1]),
            empty_squares: BitBoard::from_squares(&[Square::B1, Square::C1, Square::D1]),
        },
    ],
    [
        // Black O-O
        CastlingSetup {
            castle_side: CastleSide::KingSide,
            king_from: Square::E8,
            king_to: Square::G8,
            rook_from: Square::H8,
            rook_to: Square::F8,
            safe_squares: BitBoard::from_squares(&[Square::E8, Square::F8, Square::G8]),
            empty_squares: BitBoard::from_squares(&[Square::F8, Square::G8]),
        },
        // Black O-O-O
        CastlingSetup {
            castle_side: CastleSide::QueenSide,
            king_from: Square::E8,
            king_to: Square::C8,
            rook_from: Square::A8,
            rook_to: Square::D8,
            safe_squares: BitBoard::from_squares(&[Square::E8, Square::D8, Square::C8]),
            empty_squares: BitBoard::from_squares(&[Square::B8, Square::C8, Square::D8]),
        },
    ],
];

const KING_MOVES: [BitBoard; 64] = {
    let mut arr = [BitBoard::EMPTY; 64];
    let mut i: u8 = 0;
    while i < 64 {
        arr[i as usize] = compute_king_moves(Square(i));
        i += 1;
    }
    arr
};

const KNIGHT_ATTACKS: [BitBoard; 64] = {
    let mut arr = [BitBoard::EMPTY; 64];
    let mut i: u8 = 0;
    while i < 64 {
        arr[i as usize] = compute_knight_attacks(Square(i));
        i += 1;
    }
    arr
};

static RAYS: [[BitBoard; Dir8::COUNT]; 64] = {
    let mut arr = [[BitBoard::EMPTY; Dir8::COUNT]; 64];
    let mut i = 0;
    while i < 64 {
        arr[i] = ray_mask(i as u8);
        i += 1;
    }
    arr
};

macro_rules! ray_dir {
    ($square:expr, $($shift:ident),+) => {{
        let mut b = BitBoard::from_square(Square($square));
        let mut i = 0;
        while i < 8 {
            b = b.or(b$(.$shift())+);
            i += 1;
        }
        b
    }};
}

const fn ray_mask(square: u8) -> [BitBoard; Dir8::COUNT] {
    let origin = BitBoard::from_square(Square(square));
    [
        ray_dir!(square, shift_north).and(origin.not()),
        ray_dir!(square, shift_south).and(origin.not()),
        ray_dir!(square, shift_east).and(origin.not()),
        ray_dir!(square, shift_west).and(origin.not()),
        ray_dir!(square, shift_north, shift_east).and(origin.not()),
        ray_dir!(square, shift_south, shift_east).and(origin.not()),
        ray_dir!(square, shift_north, shift_west).and(origin.not()),
        ray_dir!(square, shift_south, shift_west).and(origin.not()),
    ]
}

const fn compute_king_moves(square: Square) -> BitBoard {
    let b = BitBoard::from_square(square);
    let mut r = b.or(b.shift_east()).or(b.shift_west());
    r = r.or(r.shift_north()).or(r.shift_south());
    r.and(b.not())
}

const fn compute_knight_attacks(square: Square) -> BitBoard {
    let b = BitBoard::from_square(square);
    let nne = b.shift_north().shift_north().shift_east();
    let nee = b.shift_north().shift_east().shift_east();
    let see = b.shift_south().shift_east().shift_east();
    let sse = b.shift_south().shift_south().shift_east();
    let ssw = b.shift_south().shift_south().shift_west();
    let sww = b.shift_south().shift_west().shift_west();
    let nww = b.shift_north().shift_west().shift_west();
    let nnw = b.shift_north().shift_north().shift_west();
    nne.or(nee).or(see).or(sse).or(ssw).or(sww).or(nww).or(nnw)
}

const fn compute_line_bb(from: Square, to: Square) -> BitBoard {
    // Determine alignment type first to avoid branching duplication
    let dir = get_dir(from, to);
    let Some(dir) = dir else {
        return BitBoard::EMPTY;
    };

    // For a full line, we backtrack to the edge in the negative direction,
    // then traverse to the opposite edge collecting squares.
    let (mut f, mut r) = (from.file() as i8, from.rank() as i8);
    let (df, dr) = match dir {
        Dir4::File => (0, 1),
        Dir4::Rank => (1, 0),
        Dir4::Diagonal => (1, 1),
        Dir4::AntiDiagonal => (1, -1),
    };

    // Step backwards to the board edge
    while (f - df) >= 0 && (f - df) < 8 && (r - dr) >= 0 && (r - dr) < 8 {
        f -= df;
        r -= dr;
    }

    // Walk forward to the opposite edge collecting all squares
    let mut bb = BitBoard::EMPTY;
    while f >= 0 && f < 8 && r >= 0 && r < 8 {
        bb = bb.set_bit(Square::from_file_rank(f as u8, r as u8).0);
        f += df;
        r += dr;
    }
    bb
}

const fn compute_between_bb(from: Square, to: Square) -> BitBoard {
    if from.0 == to.0 {
        return BitBoard::EMPTY;
    }
    let dir = get_dir(from, to);
    let Some(_dir) = dir else {
        return BitBoard::EMPTY;
    };

    let f1 = from.file() as i8;
    let r1 = from.rank() as i8;
    let f2 = to.file() as i8;
    let r2 = to.rank() as i8;

    // Compute step towards `to` using const-friendly comparisons
    let df = if f2 > f1 {
        1
    } else if f2 < f1 {
        -1
    } else {
        0
    };
    let dr = if r2 > r1 {
        1
    } else if r2 < r1 {
        -1
    } else {
        0
    };

    let mut f = f1 + df;
    let mut r = r1 + dr;
    let mut bb = BitBoard::EMPTY;
    while f != f2 || r != r2 {
        // f and r are guaranteed to be within 0..=7 for aligned squares
        bb = bb.set_bit(Square::from_file_rank(f as u8, r as u8).0);
        f += df;
        r += dr;
    }
    bb
}

#[cfg(test)]
mod tests {
    use super::*;

    fn squares(coords: &[&str]) -> Vec<Square> {
        coords
            .iter()
            .map(|s| s.parse::<Square>().unwrap())
            .collect()
    }

    #[test]
    fn rays_exclude_origin_square_in_all_directions() {
        for &sq in &["a1", "d4", "h8"] {
            let idx = sq.parse::<Square>().unwrap().0;
            for d in Dir8::ALL {
                let bb = RAYS[idx as usize][d.idx()];
                assert!(
                    bb.is_clear(idx),
                    "origin square {} (idx {}) should be clear in dir {:?}; got\n{:?}",
                    sq,
                    idx,
                    d,
                    bb
                );
            }
        }
    }

    #[test]
    fn rays_from_a1_are_correct() {
        let idx = "a1".parse::<Square>().unwrap().0;
        let cases: &[(Dir8, BitBoard)] = &[
            (
                Dir8::North,
                BitBoard::from_squares(&squares(&["a2", "a3", "a4", "a5", "a6", "a7", "a8"])),
            ),
            (
                Dir8::East,
                BitBoard::from_squares(&squares(&["b1", "c1", "d1", "e1", "f1", "g1", "h1"])),
            ),
            (Dir8::South, BitBoard::EMPTY),
            (Dir8::West, BitBoard::EMPTY),
        ];
        for &(d, expected) in cases {
            let actual = RAYS[idx as usize][d.idx()];
            assert_eq!(
                actual, expected,
                "rays from a1 in dir {:?}:\nactual:\n{:?}\nexpected:\n{:?}",
                d, actual, expected
            );
        }
    }

    #[test]
    fn rays_from_d4_are_correct() {
        let idx = "d4".parse::<Square>().unwrap().0;
        let cases: &[(Dir8, BitBoard)] = &[
            (
                Dir8::North,
                BitBoard::from_squares(&squares(&["d5", "d6", "d7", "d8"])),
            ),
            (
                Dir8::South,
                BitBoard::from_squares(&squares(&["d3", "d2", "d1"])),
            ),
            (
                Dir8::East,
                BitBoard::from_squares(&squares(&["e4", "f4", "g4", "h4"])),
            ),
            (
                Dir8::West,
                BitBoard::from_squares(&squares(&["c4", "b4", "a4"])),
            ),
        ];
        for &(d, expected) in cases {
            let actual = RAYS[idx as usize][d.idx()];
            assert_eq!(
                actual, expected,
                "rays from d4 in dir {:?}:\nactual:\n{:?}\nexpected:\n{:?}",
                d, actual, expected
            );
        }
    }

    #[test]
    fn rays_from_h8_are_correct() {
        let idx = "h8".parse::<Square>().unwrap().0;
        let cases: &[(Dir8, BitBoard)] = &[
            (Dir8::North, BitBoard::EMPTY),
            (Dir8::East, BitBoard::EMPTY),
            (
                Dir8::South,
                BitBoard::from_squares(&squares(&["h7", "h6", "h5", "h4", "h3", "h2", "h1"])),
            ),
            (
                Dir8::West,
                BitBoard::from_squares(&squares(&["g8", "f8", "e8", "d8", "c8", "b8", "a8"])),
            ),
        ];
        for &(d, expected) in cases {
            let actual = RAYS[idx as usize][d.idx()];
            assert_eq!(
                actual, expected,
                "rays from h8 in dir {:?}:\nactual:\n{:?}\nexpected:\n{:?}",
                d, actual, expected
            );
        }
    }

    #[test]
    fn knight_moves_from_a1_are_correct() {
        assert_eq!(
            compute_knight_attacks("a1".parse().unwrap()),
            BitBoard::from_squares(&squares(&["b3", "c2"]))
        );
    }
    #[test]
    fn knight_moves_from_e4_are_correct() {
        assert_eq!(
            compute_knight_attacks("e4".parse().unwrap()),
            BitBoard::from_squares(&squares(&["c3", "c5", "d2", "d6", "f2", "f6", "g3", "g5"]))
        );
    }

    #[test]
    fn knight_moves_from_h7_are_correct() {
        assert_eq!(
            compute_knight_attacks("h7".parse().unwrap()),
            BitBoard::from_squares(&squares(&["f8", "f6", "g5"]))
        );
    }
}
