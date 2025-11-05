use crate::bitboard::BitBoard;
use crate::r#move::Move;

pub struct Board {
    white: BitBoard,
    // black: BitBoard,
    // pawn: BitBoard,
    // knight: BitBoard,
    // bishop: BitBoard,
    // rook: BitBoard,
    // queen: BitBoard,
    king: BitBoard,
    white_to_move: bool,
    // TODO: also store: en passant possible?, castling possible?, side to move?,
}

impl Board {
    pub fn do_move(&mut self, m: Move) {
        self.white_to_move = !self.white_to_move;
        // clear from square and set to square
        self.king = self.king.clear_bit(m.from().0).set_bit(m.to().0);
        // update "white" bitboard
        self.white = self.white.clear_bit(m.from().0).set_bit(m.to().0);
        todo!("implement do_move")
    }

    pub fn undo_move(&mut self, m: Move, irreversible_stuff: String) {
        todo!("implement undo_move")
    }

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
            white_to_move: true,
        }
    }
}
