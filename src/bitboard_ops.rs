// Bitwise and shift operators so we can write idiomatic expressions on BitBoard
pub(crate) use crate::bitboard::BitBoard;
use crate::square::Square;

impl BitBoard {
    #[inline]
    pub const fn shl(self, v: u8) -> Self {
        BitBoard(self.0 << v)
    }
    #[inline]
    pub const fn shr(self, v: u8) -> Self {
        BitBoard(self.0 >> v)
    }
    #[inline]
    pub const fn and(self, rhs: Self) -> Self {
        BitBoard(self.0 & rhs.0)
    }
    #[inline]
    pub const fn or(self, rhs: Self) -> Self {
        BitBoard(self.0 | rhs.0)
    }
    #[inline]
    pub const fn not(self) -> Self {
        BitBoard(!self.0)
    }
}

// General BitBoard utility methods and constants
impl BitBoard {
    pub const EMPTY: BitBoard = BitBoard(0);
    pub const FULL: BitBoard = BitBoard(!0);
    pub const NOT_A_FILE: BitBoard = BitBoard(0xfefefefefefefefe);
    pub const NOT_H_FILE: BitBoard = BitBoard(0x7f7f7f7f7f7f7f7f);
    #[inline]
    pub const fn shift_north(self) -> Self {
        self.shl(8)
    }

    #[inline]
    pub const fn shift_south(self) -> Self {
        self.shr(8).and(BitBoard::NOT_A_FILE)
    }

    #[inline]
    pub const fn shift_east(self) -> Self {
        self.shl(1).and(BitBoard::NOT_A_FILE)
    }

    #[inline]
    pub const fn shift_west(self) -> Self {
        self.shr(1).and(BitBoard::NOT_H_FILE)
    }

    #[inline]
    pub fn for_each_set_bit(&self, mut f: impl FnMut(Square)) {
        let mut bb = self.0;
        while bb != 0 {
            let idx = bb.trailing_zeros() as u8;
            f(Square(idx));
            bb &= bb - 1; // clear least significant set bit
        }
    }
}
