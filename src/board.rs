pub type BB = u64;
pub type SquareIndex = u8;

struct Pieces {
    king: BB,
}

struct Board {
    white: BB,
    black: BB,
    pawn: BB,
    knight: BB,
    bishop: BB,
    rook: BB,
    queen: BB,
    king: BB,
}

fn white_kings(b: &Board) -> BB {
    b.white & b.king
}
