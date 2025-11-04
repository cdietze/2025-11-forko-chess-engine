use crate::bitboard::BitBoard;

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
