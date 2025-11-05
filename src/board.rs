use crate::bitboard::BitBoard;

pub struct Board {
    white: BitBoard,
    // black: BitBoard,
    // pawn: BitBoard,
    // knight: BitBoard,
    // bishop: BitBoard,
    // rook: BitBoard,
    // queen: BitBoard,
    king: BitBoard,
    // TODO: also store: en passant possible?, castling possible?, side to move?,
}

impl Board {
    #[inline]
    pub fn white_kings(&self) -> BitBoard {
        self.white.and(self.king)
    }

    /// Constructs a board that contains only a single white king on the given square.
    pub fn from_white_king(square: crate::square::Square) -> Self {
        let bb = BitBoard::from_square(square);
        Board {
            white: bb,
            king: bb,
        }
    }
}
