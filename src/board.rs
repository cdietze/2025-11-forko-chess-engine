use crate::bitboard::BitBoard;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct SquareIndex(pub u8);

struct Pieces {
    king: BitBoard,
}

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

fn white_kings(b: &Board) -> BitBoard {
    b.white & b.king
}
