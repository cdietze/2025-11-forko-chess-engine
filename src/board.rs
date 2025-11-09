use crate::bitboard::BitBoard;
use crate::r#move::Move;

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
    #[inline]
    pub const fn idx(self) -> usize {
        self as usize
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Board {
    /// Bitboard of all white pieces on the board, all other squares have black pieces or are empty.
    pub white: BitBoard,
    /// One BitBoard per piece type (color is derived via `white`).
    pub pieces: [BitBoard; Piece::COUNT],
    pub white_to_move: bool,
    // TODO: also store: en passant possible?, castling possible?
}

impl Board {
    pub fn color_to_move(&self) -> Color {
        if self.white_to_move {
            Color::White
        } else {
            Color::Black
        }
    }
    pub fn make_move(&mut self, m: Move) {
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
        assert!(moved_piece_idx.is_some(), "No piece found at source square");
        let pi = moved_piece_idx.unwrap();
        // Handle capture: clear any piece on destination
        for j in 0..Piece::COUNT {
            self.pieces[j] = self.pieces[j].clear_bit(to);
        }

        // Move the piece: set "to" and clear "from"
        self.pieces[pi] = self.pieces[pi].clear_bit(from).set_bit(to);

        // Update "white" BitBoard
        self.white = self.white.clear_bit(from).set(to, self.white_to_move);
        self.white_to_move = !self.white_to_move;
    }

    pub fn unmake_move(&mut self, m: Move, irreversible_stuff: String) {
        todo!("implement unmake_move")
    }

    pub fn set_piece(mut self, square: crate::square::Square, piece: Piece, color: Color) -> Self {
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

    pub fn own_color_board(&self) -> BitBoard {
        if self.white_to_move {
            self.white
        } else {
            self.white.not()
        }
    }
    pub fn color_board(&self, color: Color) -> BitBoard {
        match color {
            Color::White => self.white,
            Color::Black => self.white.not(),
        }
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
    #[inline]
    pub fn occupied(&self) -> BitBoard {
        self.pieces
            .iter()
            .copied()
            .reduce(|acc, bb| acc.or(bb))
            .unwrap_or(BitBoard(0))
    }

    #[inline]
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
        }
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
        writeln!(
            f,
            "Side to move: {}",
            if self.white_to_move { "White" } else { "Black" }
        )?;
        Ok(())
    }
}

impl Default for Board {
    fn default() -> Self {
        Board::empty()
    }
}
