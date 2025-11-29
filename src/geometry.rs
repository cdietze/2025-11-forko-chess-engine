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

pub const fn get_dir(from: Square, to: Square) -> Option<Dir4> {
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
