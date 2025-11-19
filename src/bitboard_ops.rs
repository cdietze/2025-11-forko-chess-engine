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
    pub const fn xor(self, rhs: Self) -> Self {
        BitBoard(self.0 ^ rhs.0)
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
    pub const RANK_1: BitBoard = BitBoard(0xff000000000000ff);
    pub const RANK_4: BitBoard = BitBoard(0x00000000ff000000);
    pub const RANK_5: BitBoard = BitBoard(0x000000ff00000000);
    pub const RANK_8: BitBoard = BitBoard(0xff00000000000000);
    #[inline]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
    #[inline]
    pub const fn is_not_empty(self) -> bool {
        self.0 != 0
    }
    #[inline]
    pub const fn bit_scan_forward(self) -> u8 {
        self.0.trailing_zeros() as u8
    }
    #[inline]
    pub const fn bit_scan_backward(self) -> u8 {
        63 - self.0.leading_zeros() as u8
    }
    #[inline]
    pub const fn set(self, idx: u8, value: bool) -> Self {
        if value {
            self.set_bit(idx)
        } else {
            self.clear_bit(idx)
        }
    }
    #[inline]
    pub const fn set_bit(self, idx: u8) -> Self {
        self.or(BitBoard::from_idx(idx))
    }
    #[inline]
    pub const fn clear_bit(self, idx: u8) -> Self {
        self.and(BitBoard::from_idx(idx).not())
    }
    #[inline]
    pub const fn is_set(self, idx: u8) -> bool {
        (self.0 & (1 << idx)) != 0
    }
    #[inline]
    pub const fn is_clear(self, idx: u8) -> bool {
        !self.is_set(idx)
    }
    #[inline]
    pub const fn toggle_bit(self, idx: u8) -> Self {
        self.xor(BitBoard::from_idx(idx))
    }
    #[inline]
    pub const fn shift_north(self) -> Self {
        self.shl(8)
    }
    #[inline]
    pub const fn shift_south(self) -> Self {
        self.shr(8)
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
    pub fn for_each_set_bit(&self, mut f: impl FnMut(Square) -> bool) -> bool {
        let mut bb = self.0;
        while bb != 0 {
            let idx = bb.trailing_zeros() as u8;
            if !f(Square(idx)) {
                return false;
            }
            bb &= bb - 1; // clear least significant set bit
        }
        true
    }
    #[inline]
    pub fn intersects(&self, other: BitBoard) -> bool {
        (self.0 & other.0) != 0
    }
    #[inline]
    pub fn has_square(&self, sq: Square) -> bool {
        self.is_set(sq.0)
    }
}

#[cfg(test)]
mod tests {
    use super::BitBoard;

    #[test]
    fn test_bit_scan_on_full_board() {
        let bb = BitBoard::FULL;
        assert_eq!(bb.bit_scan_forward(), 0);
        assert_eq!(bb.bit_scan_backward(), 63);
    }

    #[test]
    fn test_bit_scan_on_empty_board() {
        let bb = BitBoard::EMPTY;
        assert_eq!(bb.bit_scan_forward(), 64);
    }

    #[should_panic]
    #[test]
    fn test_bit_scan_backward_should_panic_on_empty_board() {
        let bb = BitBoard::EMPTY;
        bb.bit_scan_backward();
    }
    #[test]
    fn test_bit_scan_on_single_bit() {
        let bb = BitBoard::EMPTY.set_bit(5);
        assert_eq!(bb.bit_scan_forward(), 5);
        assert_eq!(bb.bit_scan_backward(), 5);
    }
    #[test]
    fn test_bit_scan_on_multiple_bits() {
        let bb = BitBoard::EMPTY.set_bit(5).set_bit(50);
        assert_eq!(bb.bit_scan_forward(), 5);
        assert_eq!(bb.bit_scan_backward(), 50);
    }
}
