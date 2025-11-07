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

pub struct Board {
    /// Bitboard of all white pieces on the board, all other squares have black pieces or are empty.
    pub white: BitBoard,
    /// One BitBoard per piece type (color is derived via `white`).
    pub pieces: [BitBoard; Piece::COUNT],
    pub white_to_move: bool,
    // TODO: also store: en passant possible?, castling possible?, side to move?,
}

impl Board {
    pub fn color_to_move(&self) -> Color {
        if self.white_to_move {
            Color::White
        } else {
            Color::Black
        }
    }
    pub fn do_move(&mut self, m: Move) {
        // For now, only move the white king (matches current move generator usage)
        let k = Piece::King.idx();
        self.pieces[k] = self.pieces[k].clear_bit(m.from().0).set_bit(m.to().0);
        self.white = self
            .white
            .clear_bit(m.from().0)
            .set(m.to().0, self.color_to_move() == Color::White);
        self.white_to_move = !self.white_to_move;
    }

    pub fn undo_move(&mut self, m: Move, irreversible_stuff: String) {
        todo!("implement undo_move")
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

    #[inline]
    pub fn kings(&self) -> BitBoard {
        self.pieces[Piece::King as usize]
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
                let square = rank * 8 + file;
                let is_king = self.kings().is_set(square as u8);
                let is_white = self.white.is_set(square as u8);
                let piece = if is_king {
                    if is_white { " K " } else { " k " }
                } else {
                    "   "
                };
                write!(f, "{}|", piece)?;
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
