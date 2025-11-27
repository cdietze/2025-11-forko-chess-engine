use crate::bitboard::BitBoard;
use crate::board::Color;
use crate::board::Color::White;
use crate::geometry::Dir8;
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
    // TODO: actually precompute this
    compute_knight_attacks(square)
}

#[inline]
pub const fn pawn_attacks(square: Square, color_to_move: Color, capture_east: bool) -> BitBoard {
    // TODO: actually precompute this
    compute_pawn_attacks(square, color_to_move, capture_east)
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

const fn compute_pawn_attacks(
    square: Square,
    color_to_move: Color,
    capture_east: bool,
) -> BitBoard {
    let b = BitBoard::from_square(square);
    let b = match color_to_move {
        White => b.shift_north(),
        _ => b.shift_south(),
    };
    let b = if capture_east {
        b.shift_east()
    } else {
        b.shift_west()
    };
    b
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
    #[test]
    fn white_pawn_attacks_from_d4_are_correct() {
        assert_eq!(
            compute_pawn_attacks("d4".parse().unwrap(), White, true),
            BitBoard::from_squares(&squares(&["e5"]))
        );
        assert_eq!(
            compute_pawn_attacks("d4".parse().unwrap(), White, false),
            BitBoard::from_squares(&squares(&["c5"]))
        );
    }

    #[test]
    fn black_pawn_attacks_from_d4_are_correct() {
        assert_eq!(
            compute_pawn_attacks("d4".parse().unwrap(), Color::Black, true),
            BitBoard::from_squares(&squares(&["e3"]))
        );
        assert_eq!(
            compute_pawn_attacks("d4".parse().unwrap(), Color::Black, false),
            BitBoard::from_squares(&squares(&["c3"]))
        );
    }
}
