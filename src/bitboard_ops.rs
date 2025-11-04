// Bitwise and shift operators so we can write idiomatic expressions on BitBoard
use crate::bitboard::BitBoard;
use core::ops::{
    BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not, Shl, ShlAssign, Shr,
    ShrAssign,
};

impl BitAnd for BitBoard {
    type Output = Self;
    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        BitBoard(self.0 & rhs.0)
    }
}

impl BitAndAssign for BitBoard {
    #[inline]
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl BitOr for BitBoard {
    type Output = Self;
    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        BitBoard(self.0 | rhs.0)
    }
}

impl BitOrAssign for BitBoard {
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitXor for BitBoard {
    type Output = Self;
    #[inline]
    fn bitxor(self, rhs: Self) -> Self::Output {
        BitBoard(self.0 ^ rhs.0)
    }
}

impl BitXorAssign for BitBoard {
    #[inline]
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0;
    }
}

impl Not for BitBoard {
    type Output = Self;
    #[inline]
    fn not(self) -> Self::Output {
        BitBoard(!self.0)
    }
}

// Shifts: implement for u32 and u8 to make using integer literals ergonomic
impl Shl<u32> for BitBoard {
    type Output = Self;
    #[inline]
    fn shl(self, rhs: u32) -> Self::Output {
        BitBoard(self.0 << rhs)
    }
}

impl Shr<u32> for BitBoard {
    type Output = Self;
    #[inline]
    fn shr(self, rhs: u32) -> Self::Output {
        BitBoard(self.0 >> rhs)
    }
}

impl Shl<u8> for BitBoard {
    type Output = Self;
    #[inline]
    fn shl(self, rhs: u8) -> Self::Output {
        BitBoard(self.0 << rhs)
    }
}

impl Shr<u8> for BitBoard {
    type Output = Self;
    #[inline]
    fn shr(self, rhs: u8) -> Self::Output {
        BitBoard(self.0 >> rhs)
    }
}

impl ShlAssign<u32> for BitBoard {
    #[inline]
    fn shl_assign(&mut self, rhs: u32) {
        self.0 <<= rhs;
    }
}

impl ShrAssign<u32> for BitBoard {
    #[inline]
    fn shr_assign(&mut self, rhs: u32) {
        self.0 >>= rhs;
    }
}

impl ShlAssign<u8> for BitBoard {
    #[inline]
    fn shl_assign(&mut self, rhs: u8) {
        self.0 <<= rhs;
    }
}

impl ShrAssign<u8> for BitBoard {
    #[inline]
    fn shr_assign(&mut self, rhs: u8) {
        self.0 >>= rhs;
    }
}

// General BitBoard utility methods and constants
impl BitBoard {
    pub const NOT_A_FILE: BitBoard = BitBoard(0xfefefefefefefefe);
    pub const NOT_H_FILE: BitBoard = BitBoard(0x7f7f7f7f7f7f7f7f);
    #[inline]
    pub fn shift_north(self) -> Self {
        self << 8u32
    }

    #[inline]
    pub fn shift_south(self) -> Self {
        self >> 8u32
    }

    #[inline]
    pub fn shift_east(self) -> Self {
        (self << 1u32) & BitBoard::NOT_A_FILE
    }

    #[inline]
    pub fn shift_west(self) -> Self {
        (self >> 1u32) & BitBoard::NOT_H_FILE
    }
}
