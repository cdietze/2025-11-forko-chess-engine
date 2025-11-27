use crate::bitboard::BitBoard;
use crate::square::Square;

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Dir4 {
    Rank,
    File,
    Diagonal,
    AntiDiagonal,
}
#[allow(dead_code)]
impl Dir4 {
    #[allow(dead_code)]
    pub const COUNT: usize = 4;
    pub const ALL: [Dir4; Dir4::COUNT] =
        [Dir4::Rank, Dir4::File, Dir4::Diagonal, Dir4::AntiDiagonal];
    #[inline]
    pub const fn idx(self) -> usize {
        self as usize
    }
}
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Dir8 {
    North,
    South,
    East,
    West,
    NorthEast,
    SouthEast,
    NorthWest,
    SouthWest,
}
impl Dir8 {
    pub const COUNT: usize = 8;
    #[allow(dead_code)]
    pub const ALL: [Dir8; Dir8::COUNT] = [
        Dir8::North,
        Dir8::South,
        Dir8::East,
        Dir8::West,
        Dir8::NorthEast,
        Dir8::SouthEast,
        Dir8::NorthWest,
        Dir8::SouthWest,
    ];
    #[inline]
    pub const fn idx(self) -> usize {
        self as usize
    }
}

/// Returns a `BitBoard` containing all squares that lie on a complete line of two given squares.
/// When from and two are not on a same line, rank, diagonal or anti diagonal, an empty bitboard is returned.
/// Otherwise the result contains all 8 squares of the line, rank, diagonal or anti diagonal on which
/// `from` and `to` lie.
/// TODO: precompute in 2d array
pub fn line_bb(from: Square, to: Square) -> BitBoard {
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
    while (0..8).contains(&f) && (0..8).contains(&r) {
        bb = bb.set_bit(Square::from_file_rank(f as u8, r as u8).0);
        f += df;
        r += dr;
    }
    bb
}

/// Calculates the set of squares that lie between two given chessboard squares as a `BitBoard`
/// (exclusive of the destination square), considering alignment along files, ranks, or diagonals.
///
/// If the two squares are not aligned (i.e., they do not lie on the same file, rank, or diagonal),
/// an empty `BitBoard` is returned, as there are no obstructed squares along the line between the
/// two points.
/// TODO: precompute in 2d array
pub fn between_bb(from: Square, to: Square) -> BitBoard {
    // Use same alignment logic as lineBB to keep things DRY
    if from.0 == to.0 {
        return BitBoard::EMPTY;
    }
    let dir = get_dir(from, to);
    let Some(dir) = dir else {
        return BitBoard::EMPTY;
    };

    let (f1, r1) = (from.file() as i8, from.rank() as i8);
    let (f2, r2) = (to.file() as i8, to.rank() as i8);

    // Step direction must point from 'from' towards 'to'
    let (df, dr) = match dir {
        Dir4::File => (0, (r2 - r1).signum()),
        Dir4::Rank => ((f2 - f1).signum(), 0),
        Dir4::Diagonal | Dir4::AntiDiagonal => ((f2 - f1).signum(), (r2 - r1).signum()),
    };

    let mut bb = BitBoard::EMPTY;
    // Start from first square after 'from'
    let mut f = f1 + df;
    let mut r = r1 + dr;
    while f != f2 || r != r2 {
        bb = bb.set_bit(Square::from_file_rank(f as u8, r as u8).0);
        f += df;
        r += dr;
    }
    bb
}

pub fn get_dir(from: Square, to: Square) -> Option<Dir4> {
    let f1 = from.file();
    let r1 = from.rank();
    let f2 = to.file();
    let r2 = to.rank();

    if f1 == f2 {
        return Some(Dir4::File);
    }
    if r1 == r2 {
        return Some(Dir4::Rank);
    }
    let fd = f1 as i8 - f2 as i8;
    let rd = r1 as i8 - r2 as i8;
    if fd.abs() == rd.abs() {
        // Diagonal or anti-diagonal
        if (fd > 0 && rd > 0) || (fd < 0 && rd < 0) {
            Some(Dir4::Diagonal)
        } else {
            Some(Dir4::AntiDiagonal)
        }
    } else {
        None
    }
}
