use crate::bitboard::BitBoard;
use crate::square::Square;

/// Precomputed king move bitboards
pub const KING_MOVES: [BitBoard; 64] = {
    let mut arr = [BitBoard::EMPTY; 64];
    let mut i = 0;
    while i < 64 {
        let bb = BitBoard::from_square(Square(i as u8));
        arr[i] = bb.king_moves();
        i += 1;
    }
    arr
};

/// See https://www.chessprogramming.org/Classical_Approach
pub static RAYS: [Rays; 64] = {
    let mut arr = [Rays {
        north: BitBoard::EMPTY,
        south: BitBoard::EMPTY,
        east: BitBoard::EMPTY,
        west: BitBoard::EMPTY,
        north_east: BitBoard::EMPTY,
        south_east: BitBoard::EMPTY,
        north_west: BitBoard::EMPTY,
        south_west: BitBoard::EMPTY,
    }; 64];
    let mut i = 0;
    while i < 64 {
        arr[i] = ray_mask(i as u8);
        i += 1;
    }
    arr
};

#[derive(Copy, Clone)]
pub struct Rays {
    pub north: BitBoard,
    pub south: BitBoard,
    pub east: BitBoard,
    pub west: BitBoard,
    pub north_east: BitBoard,
    pub south_east: BitBoard,
    pub north_west: BitBoard,
    pub south_west: BitBoard,
}

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
const fn ray_mask(square: u8) -> Rays {
    let origin = BitBoard::from_square(Square(square));
    fn shift_ne(b: BitBoard) -> BitBoard {
        b.shift_north().shift_east()
    }
    Rays {
        north: ray_dir!(square, shift_north).and(origin.not()),
        south: ray_dir!(square, shift_south).and(origin.not()),
        east: ray_dir!(square, shift_east).and(origin.not()),
        west: ray_dir!(square, shift_west).and(origin.not()),
        north_east: ray_dir!(square, shift_north, shift_east).and(origin.not()),
        south_east: ray_dir!(square, shift_south, shift_east).and(origin.not()),
        north_west: ray_dir!(square, shift_north, shift_west).and(origin.not()),
        south_west: ray_dir!(square, shift_south, shift_west).and(origin.not()),
    }
}

impl BitBoard {
    const fn king_moves(self) -> BitBoard {
        let b = self;
        let mut r = b.or(b.shift_east()).or(b.shift_west());
        r = r.or(r.shift_north()).or(r.shift_south());
        r.and(b.not())
    }
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
            let rays = RAYS[idx as usize];
            assert!(
                rays.north.is_clear(idx),
                "north should exclude origin for {}",
                sq
            );
            assert!(
                rays.south.is_clear(idx),
                "south should exclude origin for {}",
                sq
            );
            assert!(
                rays.east.is_clear(idx),
                "east should exclude origin for {}",
                sq
            );
            assert!(
                rays.west.is_clear(idx),
                "west should exclude origin for {}",
                sq
            );
        }
    }

    #[test]
    fn rays_from_a1_are_correct() {
        let idx = "a1".parse::<Square>().unwrap().0;
        let rays = RAYS[idx as usize];
        let expected_north = bb_from_coords(&["a2", "a3", "a4", "a5", "a6", "a7", "a8"]);
        let expected_east = bb_from_coords(&["b1", "c1", "d1", "e1", "f1", "g1", "h1"]);
        let expected_south = BitBoard::EMPTY; // beyond bottom edge
        let expected_west = BitBoard::EMPTY; // beyond a-file

        assert_eq!(
            rays.north, expected_north,
            "north ray from a1 incorrect\nactual:\n{:?}\nexpected:\n{:?}",
            rays.north, expected_north
        );
        assert_eq!(
            rays.east, expected_east,
            "east ray from a1 incorrect\nactual:\n{:?}\nexpected:\n{:?}",
            rays.east, expected_east
        );
        assert_eq!(
            rays.south, expected_south,
            "south ray from a1 should be empty"
        );
        assert_eq!(rays.west, expected_west, "west ray from a1 should be empty");
    }

    #[test]
    fn rays_from_d4_are_correct() {
        let idx = "d4".parse::<Square>().unwrap().0;
        let rays = RAYS[idx as usize];

        let expected_north = bb_from_coords(&["d5", "d6", "d7", "d8"]);
        let expected_south = bb_from_coords(&["d3", "d2", "d1"]);
        let expected_east = bb_from_coords(&["e4", "f4", "g4", "h4"]);
        let expected_west = bb_from_coords(&["c4", "b4", "a4"]);

        assert_eq!(
            rays.north, expected_north,
            "north ray from d4 incorrect\nactual:\n{:?}\nexpected:\n{:?}",
            rays.north, expected_north
        );
        assert_eq!(
            rays.south, expected_south,
            "south ray from d4 incorrect\nactual:\n{:?}\nexpected:\n{:?}",
            rays.south, expected_south
        );
        assert_eq!(
            rays.east, expected_east,
            "east ray from d4 incorrect\nactual:\n{:?}\nexpected:\n{:?}",
            rays.east, expected_east
        );
        assert_eq!(
            rays.west, expected_west,
            "west ray from d4 incorrect\nactual:\n{:?}\nexpected:\n{:?}",
            rays.west, expected_west
        );
    }

    #[test]
    fn rays_from_h8_are_correct() {
        let idx = "h8".parse::<Square>().unwrap().0;
        let rays = RAYS[idx as usize];

        let expected_north = BitBoard::EMPTY; // nothing beyond top edge
        let expected_east = BitBoard::EMPTY; // nothing beyond h-file
        let expected_south = bb_from_coords(&["h7", "h6", "h5", "h4", "h3", "h2", "h1"]);
        let expected_west = bb_from_coords(&["g8", "f8", "e8", "d8", "c8", "b8", "a8"]);

        assert_eq!(
            rays.north, expected_north,
            "north ray from h8 should be empty"
        );
        assert_eq!(rays.east, expected_east, "east ray from h8 should be empty");
        assert_eq!(
            rays.south, expected_south,
            "south ray from h8 incorrect\nactual:\n{:?}\nexpected:\n{:?}",
            rays.south, expected_south
        );
        assert_eq!(
            rays.west, expected_west,
            "west ray from h8 incorrect\nactual:\n{:?}\nexpected:\n{:?}",
            rays.west, expected_west
        );
    }
}
