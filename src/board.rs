use crate::bitboard::BitBoard;
use crate::r#move::Move;
use crate::move_gen::is_legal;
use crate::precomputed::CastleSide::{KingSide, QueenSide};
use crate::precomputed::{CASTLING_SETUPS, CastleSide};
use crate::square::Square;

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Color {
    White = 0,
    Black = 1,
}

impl Color {
    pub const COUNT: usize = 2;
    #[inline]
    pub const fn idx(self) -> usize {
        self as usize
    }
    #[inline]
    pub const fn opposite(self) -> Self {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
    #[inline]
    pub fn forward_offset(self) -> i8 {
        match self {
            Color::White => 8,
            Color::Black => -8,
        }
    }
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Color::White => write!(f, "White"),
            Color::Black => write!(f, "Black"),
        }
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Piece {
    King = 0,
    Queen = 1,
    Rook = 2,
    Bishop = 3,
    Knight = 4,
    Pawn = 5,
}

impl Piece {
    pub const COUNT: usize = 6;

    pub const ALL: [Piece; Piece::COUNT] = [
        Piece::King,
        Piece::Queen,
        Piece::Rook,
        Piece::Bishop,
        Piece::Knight,
        Piece::Pawn,
    ];
    #[inline]
    pub const fn idx(self) -> usize {
        self as usize
    }
}

/// Flags for one side whether castling is allowed or not (kingside, queenside).
pub type CastlingRights = [bool; CastleSide::COUNT];

#[derive(Copy, Clone, Debug)]
pub struct Board {
    /// Bitboard of all white pieces on the board, all other squares have black pieces or are empty.
    pub white: BitBoard,
    /// One BitBoard per piece type (color is derived via `white`).
    pub pieces: [BitBoard; Piece::COUNT],
    pub white_to_move: bool,
    /// The square where a e.p. pawn can be captured. Or `Square::ILLEGAL_SQUARE` if no e.p. is possible.
    pub en_passant: Square,
    pub castling_rights: [CastlingRights; Color::COUNT],
}

impl Board {
    #[inline]
    pub fn color_to_move(&self) -> Color {
        if self.white_to_move {
            Color::White
        } else {
            Color::Black
        }
    }
    pub fn make_move(&mut self, m: Move) -> Result<(), String> {
        // println!("make_move, board:\n{}", self);
        // println!(
        //     "make_move: {:?} from: {}({}), to: {}({})",
        //     m,
        //     m.from(),
        //     m.from().0,
        //     m.to(),
        //     m.to().0
        // );
        let from = m.from().0;
        let to = m.to().0;

        // Determine which piece is moving based on the source square
        let mut moved_piece_idx: Option<usize> = None;
        for i in 0..Piece::COUNT {
            if self.pieces[i].is_set(from) {
                moved_piece_idx = Some(i);
                break;
            }
        }
        debug_assert!(moved_piece_idx.is_some(), "No piece found at source square");
        let pi = moved_piece_idx.unwrap();
        // Handle capture: clear any piece on destination
        if m.capture() {
            for j in 0..Piece::COUNT {
                self.pieces[j] = self.pieces[j].clear_bit(to);
            }
        }

        if m.promotion() {
            // Promote the piece: clear "from" and add pomotion piece to "to"
            self.pieces[pi] = self.pieces[pi].clear_bit(from);
            self.pieces[m.promotion_piece().idx()] =
                self.pieces[m.promotion_piece().idx()].set_bit(to);
        } else {
            // Move the piece: set "to" and clear "from"
            self.pieces[pi] = self.pieces[pi].clear_bit(from).set_bit(to);
        }

        if !m.promotion() && m.special0() {
            // It's a castling move. At this point the king has already been moved
            // by the generic movement code above. We only need to move the rook
            // and update color bitboards and castling rights.
            let side: CastleSide = if m.special1() { QueenSide } else { KingSide };
            let color_idx = self.color_to_move().idx();
            let setup = &CASTLING_SETUPS[color_idx][side as usize];

            // Move the rook from its original square to the destination square
            let rook_idx = Piece::Rook.idx();
            self.pieces[rook_idx] = self.pieces[rook_idx]
                .clear_bit(setup.rook_from.0)
                .set_bit(setup.rook_to.0);

            // Update color bitboard for the rook move
            self.white = self
                .white
                .set(setup.rook_from.0, false)
                .set(setup.rook_to.0, self.white_to_move);
        }

        // Clear the en passant square
        self.en_passant = Square::ILLEGAL_SQUARE;
        if pi == Piece::Pawn.idx() && !m.promotion() {
            if !m.capture() && m.special1() {
                // This is a double pawn push: Spawn a en passant pawn at the skipped square
                self.en_passant = m.from().add_offset(self.color_to_move().forward_offset());
            } else if m.capture() && m.special1() {
                // This is a en passant capture, remove the pawn
                let actual_pawn_square = m.to().add_offset(-self.color_to_move().forward_offset());
                self.pieces[Piece::Pawn.idx()] =
                    self.pieces[Piece::Pawn.idx()].clear_bit(actual_pawn_square.0);
            }
        }

        if pi == Piece::King.idx() {
            // After the king has moved, both castling rights for that color are gone
            self.castling_rights[self.color_to_move().idx()] = [false, false];
        }
        if pi == Piece::Rook.idx() {
            // After the rook has moved, remove castling rights for that castle
            let setups = &CASTLING_SETUPS[self.color_to_move().idx()];
            for setup in setups {
                if setup.rook_from == m.from() {
                    self.castling_rights[self.color_to_move().idx()][setup.castle_side.idx()] =
                        false;
                }
            }
        }
        if m.capture() {
            // If a rook is captured, castling rights for that castle are lost as well.
            let opp_color = self.color_to_move().opposite();
            let setups = &CASTLING_SETUPS[opp_color.idx()];
            for setup in setups {
                if setup.rook_from == m.to() {
                    self.castling_rights[opp_color.idx()][setup.castle_side.idx()] = false;
                }
            }
        }
        // Update "white" BitBoard: update "to" and "from" does not matter anymore,
        self.white = self.white.set(to, self.white_to_move);
        self.white_to_move = !self.white_to_move;
        if !is_legal(self) {
            return Err("Illegal move".to_string());
        }
        Ok(())
    }

    pub fn unmake_move(&mut self, _m: Move, _irreversible_stuff: String) {
        todo!("implement unmake_move")
    }

    pub fn set_piece(mut self, square: Square, piece: Piece, color: Color) -> Self {
        match color {
            Color::White => self.white = self.white.set_bit(square.0),
            Color::Black => self.white = self.white.clear_bit(square.0),
        }
        let p = piece.idx();
        self.pieces[p] = self.pieces[p].set_bit(square.0);
        self
    }

    pub fn set_color_to_move(mut self, color: Color) -> Self {
        self.white_to_move = color == Color::White;
        self
    }

    #[inline]
    pub fn kings(&self) -> BitBoard {
        self.pieces[Piece::King as usize]
    }

    pub fn color_board(&self, color: Color) -> BitBoard {
        match color {
            Color::White => self.white,
            Color::Black => self.white.not(),
        }
    }
    pub fn own_color_board(&self) -> BitBoard {
        self.color_board(self.color_to_move())
    }

    pub fn king_square(&self, color: Color) -> Square {
        Square(self.kings().and(self.color_board(color)).bit_scan_forward())
    }

    #[inline]
    pub fn white_kings(&self) -> BitBoard {
        self.white.and(self.kings())
    }

    #[inline]
    pub fn pieces(&self, piece: Piece, color: Color) -> BitBoard {
        let bb = self.pieces[piece.idx()];
        match color {
            Color::White => bb.and(self.white),
            Color::Black => bb.and(self.white.not()),
        }
    }

    /// Returns a bitboard of all occupied squares
    pub fn occupied(&self) -> BitBoard {
        self.pieces
            .iter()
            .copied()
            .reduce(|acc, bb| acc.or(bb))
            .unwrap_or(BitBoard(0))
    }

    fn piece_at(&self, square: u8) -> Option<(Piece, Color)> {
        if !self.occupied().is_set(square) {
            return None;
        }
        let color = if self.white.is_set(square) {
            Color::White
        } else {
            Color::Black
        };
        for i in 0..Piece::COUNT {
            if self.pieces[i].is_set(square) {
                let piece = match i {
                    0 => Piece::King,
                    1 => Piece::Queen,
                    2 => Piece::Rook,
                    3 => Piece::Bishop,
                    4 => Piece::Knight,
                    5 => Piece::Pawn,
                    _ => unreachable!(),
                };
                return Some((piece, color));
            }
        }
        None
    }

    /// Creates an empty board with no pieces set.
    #[inline]
    pub const fn empty() -> Self {
        Board {
            white: BitBoard(0),
            pieces: [BitBoard(0); Piece::COUNT],
            white_to_move: true,
            en_passant: Square::ILLEGAL_SQUARE,
            castling_rights: [[true; 2]; Color::COUNT],
        }
    }

    pub fn normalize(mut self) -> Self {
        let mut rights = self.castling_rights;
        // Validate castling rights against actual piece placement.
        // Kings must be on e1/e8 and corresponding rooks on h1/a1/h8/a8.
        let white_kings = self.pieces(Piece::King, Color::White);
        let white_rooks = self.pieces(Piece::Rook, Color::White);
        let black_kings = self.pieces(Piece::King, Color::Black);
        let black_rooks = self.pieces(Piece::Rook, Color::Black);

        // White kingside: King on E1 and rook on H1
        if rights[Color::White.idx()][0] {
            if !(white_kings.is_set(Square::E1.0) && white_rooks.is_set(Square::H1.0)) {
                rights[Color::White.idx()][0] = false;
            }
        }
        // White queenside: King on E1 and rook on A1
        if rights[Color::White.idx()][1] {
            if !(white_kings.is_set(Square::E1.0) && white_rooks.is_set(Square::A1.0)) {
                rights[Color::White.idx()][1] = false;
            }
        }
        // Black kingside: King on E8 and rook on H8
        if rights[Color::Black.idx()][0] {
            if !(black_kings.is_set(Square::E8.0) && black_rooks.is_set(Square::H8.0)) {
                rights[Color::Black.idx()][0] = false;
            }
        }
        // Black queenside: King on E8 and rook on A8
        if rights[Color::Black.idx()][1] {
            if !(black_kings.is_set(Square::E8.0) && black_rooks.is_set(Square::A8.0)) {
                rights[Color::Black.idx()][1] = false;
            }
        }
        self.castling_rights = rights;
        self
    }
}

impl std::fmt::Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "  +---+---+---+---+---+---+---+---+")?;
        for rank in (0..8).rev() {
            write!(f, "{} |", rank + 1)?;
            for file in 0..8 {
                let square = (rank * 8 + file) as u8;
                let cell = if let Some((piece, color)) = self.piece_at(square) {
                    let ch = match piece {
                        Piece::King => 'K',
                        Piece::Queen => 'Q',
                        Piece::Rook => 'R',
                        Piece::Bishop => 'B',
                        Piece::Knight => 'N',
                        Piece::Pawn => 'P',
                    };
                    if color == Color::White {
                        format!(" {} ", ch)
                    } else {
                        format!(" {} ", ch.to_ascii_lowercase())
                    }
                } else {
                    "   ".to_string()
                };
                write!(f, "{}|", cell)?;
            }
            writeln!(f)?;
            writeln!(f, "  +---+---+---+---+---+---+---+---+")?;
        }
        writeln!(f, "    a   b   c   d   e   f   g   h")?;

        let fmt_castling = |color: Color| -> String {
            let rights = &self.castling_rights[color.idx()];
            let mut options: Vec<&str> = Vec::new();
            if rights[KingSide.idx()] {
                options.push("O-O");
            }
            if rights[QueenSide.idx()] {
                options.push("O-O-O");
            }
            if options.is_empty() {
                "none".to_string()
            } else {
                options.join(" ")
            }
        };
        writeln!(f, "Castling white: {}", fmt_castling(Color::White))?;
        writeln!(f, "Castling black: {}", fmt_castling(Color::Black))?;
        write!(f, "En passant: ")?;
        if self.en_passant == Square::ILLEGAL_SQUARE {
            writeln!(f, "none")?;
        } else {
            writeln!(f, "{}", self.en_passant)?;
        }
        writeln!(f, "Side to move: {}", self.color_to_move())?;
        Ok(())
    }
}

impl Default for Board {
    fn default() -> Self {
        Board::empty()
    }
}

#[cfg(test)]
mod tests {
    use crate::board::{Board, Color, Piece};
    use crate::r#move::Move;
    use crate::precomputed::CastleSide::{KingSide, QueenSide};
    use crate::square::Square;

    #[test]
    fn white_castling_should_move_pieces_correctly() {
        let board = Board::from_fen("8/8/8/8/8/4k3/P6P/R3K2R w KQ - 0 1");
        let mut board1 = board;
        board1
            .make_move(Move::new_castle(Square::E1, Square::G1, KingSide))
            .unwrap();
        println!("board after king-side castle:\n{}", board1);
        let mut board2 = board;
        board2
            .make_move(Move::new_castle(Square::E1, Square::C1, QueenSide))
            .unwrap();
        println!("board after queen-side castle:\n{}", board2);
    }

    #[test]
    fn en_passant_capture_should_move_pieces_correctly() {
        let mut board = Board::from_fen("7k/8/7K/8/1p6/8/P7/8 w - - 0 1");
        board
            .make_move(Move::new_double_pawn_push(Square::A2, Square::A4))
            .unwrap();
        board
            .make_move(Move::new_en_passant(Square::B4, Square::A3))
            .unwrap();
        println!("board after en passant capture:\n{}", board);
    }

    #[test]
    fn promotion_should_work() {
        let mut board = Board::from_fen("5k1K/2P5/8/8/8/8/8/8 w - - 0 1");
        let r = board.make_move(Move::new_promotion(
            Square::C7,
            Square::C8,
            false,
            Piece::Queen,
        ));
        assert_eq!(
            board
                .pieces(Piece::Queen, Color::White)
                .is_set(Square::C8.0),
            true
        );
    }
}
