use crate::bitboard::BitBoard;
use crate::board::Color;
use crate::board::Color::White;
use crate::geometry::Dir8;
use crate::square::Square;

#[inline]
pub const fn king_moves(square: Square) -> BitBoard {
    KING_MOVES[square.0 as usize]
}

#[inline]
pub const fn ray_attacks(square: Square, dir: Dir8) -> BitBoard {
    RAYS[square.0 as usize][dir.idx()]
}

#[inline]
pub const fn pawn_attacks(square: Square, color_to_move: Color, capture_east: bool) -> BitBoard {
    // TODO: actually precompute this
    compute_pawn_attacks(square, color_to_move, capture_east)
}

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

mod tests {
    use super::*;

    fn bb_from_coords(coords: &[&str]) -> BitBoard {
        BitBoard::try_from_coords(coords.iter().cloned()).unwrap()
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
                bb_from_coords(&["a2", "a3", "a4", "a5", "a6", "a7", "a8"]),
            ),
            (
                Dir8::East,
                bb_from_coords(&["b1", "c1", "d1", "e1", "f1", "g1", "h1"]),
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
            (Dir8::North, bb_from_coords(&["d5", "d6", "d7", "d8"])),
            (Dir8::South, bb_from_coords(&["d3", "d2", "d1"])),
            (Dir8::East, bb_from_coords(&["e4", "f4", "g4", "h4"])),
            (Dir8::West, bb_from_coords(&["c4", "b4", "a4"])),
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
                bb_from_coords(&["h7", "h6", "h5", "h4", "h3", "h2", "h1"]),
            ),
            (
                Dir8::West,
                bb_from_coords(&["g8", "f8", "e8", "d8", "c8", "b8", "a8"]),
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
    fn white_pawn_attacks_from_d4_are_correct() {
        assert_eq!(
            compute_pawn_attacks("d4".parse().unwrap(), White, true),
            bb_from_coords(&["e5"])
        );
        assert_eq!(
            compute_pawn_attacks("d4".parse().unwrap(), White, false),
            bb_from_coords(&["c5"])
        );
    }

    #[test]
    fn black_pawn_attacks_from_d4_are_correct() {
        assert_eq!(
            compute_pawn_attacks("d4".parse().unwrap(), Color::Black, true),
            bb_from_coords(&["e3"])
        );
        assert_eq!(
            compute_pawn_attacks("d4".parse().unwrap(), Color::Black, false),
            bb_from_coords(&["c3"])
        );
    }
}
