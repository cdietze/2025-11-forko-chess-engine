use crate::bitboard::BitBoard;

struct Board {
    white: BitBoard,
    black: BitBoard,
    pawn: BitBoard,
    knight: BitBoard,
    bishop: BitBoard,
    rook: BitBoard,
    queen: BitBoard,
    king: BitBoard,
}

impl Board {
    #[inline]
    fn white_kings(&self) -> BitBoard {
        self.white & self.king
    }
}
