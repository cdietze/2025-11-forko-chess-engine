use crate::bitboard::BitBoard;

pub struct Board {
    white: BitBoard,
    black: BitBoard,
    pawn: BitBoard,
    knight: BitBoard,
    bishop: BitBoard,
    rook: BitBoard,
    queen: BitBoard,
    king: BitBoard,
    // TODO: also store: en passant possible?, castling possible?, side to move?,
}

impl Board {
    #[inline]
    fn white_kings(&self) -> BitBoard {
        self.white & self.king
    }
}
