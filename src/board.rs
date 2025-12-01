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

    #[inline]
    fn from_idx(i: usize) -> Piece {
        match i {
            0 => Piece::King,
            1 => Piece::Queen,
            2 => Piece::Rook,
            3 => Piece::Bishop,
            4 => Piece::Knight,
            _ => Piece::Pawn,
        }
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

#[derive(Copy, Clone, Debug)]
pub struct UnmakeInfo {
    /// En-passant square before the move (Square::ILLEGAL_SQUARE if none)
    pub prev_en_passant: Square,
    /// Castling rights before the move
    pub prev_castling_rights: [CastlingRights; Color::COUNT],
    /// Captured piece (if any) of the move being undone. None for quiet moves and en-passant is
    /// represented as Some(Pawn).
    pub captured_piece: Option<Piece>,
    /// The moved piece type
    pub moved_piece: Piece,
}

impl Board {
    /// Perform a move and return the information required to unmake it efficiently.
    /// This is a thin wrapper around make_move that captures the previous irreversible state
    /// and the captured piece (if any).
    pub fn make_move_with_info(&mut self, m: Move) -> Result<UnmakeInfo, String> {
        // Apply the move and get unmake info
        let info = self.make_move_unchecked(m);

        // Check legality
        if !is_legal(self) {
            // Restore board to original state
            self.unmake_move(m, info);
            return Err("Illegal move".to_string());
        }

        Ok(info)
    }
    #[inline]
    pub fn color_to_move(&self) -> Color {
        if self.white_to_move {
            Color::White
        } else {
            Color::Black
        }
    }
    pub fn make_move(&mut self, m: Move) -> Result<(), String> {
        // Use make_move_unchecked and check legality afterwards
        let info = self.make_move_unchecked(m);

        if !is_legal(self) {
            // Restore board to original state
            self.unmake_move(m, info);
            return Err("Illegal move".to_string());
        }

        Ok(())
    }

    /// Apply a move without checking legality. Returns UnmakeInfo for rollback.
    pub fn make_move_unchecked(&mut self, m: Move) -> UnmakeInfo {
        let from = m.from().0;
        let to = m.to().0;

        // Save irreversible state
        let prev_en_passant = self.en_passant;
        let prev_castling_rights = self.castling_rights;

        // Find moved piece index (0..=5) by probing our six piece bitboards
        let mut pi = 0usize;
        while pi < Piece::COUNT && !self.pieces[pi].is_set(from) {
            pi += 1;
        }
        debug_assert!(pi < Piece::COUNT, "No piece at source square");
        let moved_piece = Piece::from_idx(pi);

        // Figure out captured piece (if any) cheaply
        let captured_piece = if m.capture() {
            if moved_piece == Piece::Pawn && !m.promotion() && m.special1() {
                Some(Piece::Pawn) // en passant
            } else {
                let mut ci = 0usize;
                let mut found = None;
                while ci < Piece::COUNT {
                    if self.pieces[ci].is_set(to) {
                        found = Some(Piece::from_idx(ci));
                        break;
                    }
                    ci += 1;
                }
                found
            }
        } else {
            None
        };

        // Execute the move on piece bitboards
        if m.promotion() {
            // Pawn promotes to piece encoded in move
            self.pieces[pi] = self.pieces[pi].clear_bit(from);
            let pr = m.promotion_piece().idx();
            // If it’s a capture, clear only the captured piece on 'to'
            if captured_piece.is_some() {
                let ci = m.promotion_piece().idx(); // dummy use to keep pr separate
                let _ = ci; // silence unused in debug when not capturing
                // Clear destination occupant (detected above)
                if let Some(cp) = captured_piece {
                    self.pieces[cp.idx()] = self.pieces[cp.idx()].clear_bit(to);
                }
            }
            self.pieces[pr] = self.pieces[pr].set_bit(to);
        } else {
            // Normal move (quiet or capture, including castle king leg and ep destination)
            if let Some(cp) = captured_piece {
                // Normal capture (not en passant)
                if !(m.special1() && moved_piece == Piece::Pawn) {
                    self.pieces[cp.idx()] = self.pieces[cp.idx()].clear_bit(to);
                }
            }
            // Move the mover
            self.pieces[pi].0 ^= (1u64 << from) | (1u64 << to);
        }

        // Special moves
        if !m.promotion() && m.special0() {
            // Castling: move rook
            let side: CastleSide = if m.special1() { QueenSide } else { KingSide };
            let setup = &CASTLING_SETUPS[self.color_to_move().idx()][side as usize];
            let rook = Piece::Rook.idx();
            self.pieces[rook].0 ^= (1u64 << setup.rook_from.0) | (1u64 << setup.rook_to.0);
            // Update color map for rook squares (both occupied after move)
            self.white = self
                .white
                .set(setup.rook_from.0, false)
                .set(setup.rook_to.0, self.white_to_move);
        }

        // En passant housekeeping
        self.en_passant = Square::ILLEGAL_SQUARE;
        if moved_piece == Piece::Pawn {
            if !m.promotion() && !m.capture() && m.special1() {
                // double push: enable ep square behind pawn
                self.en_passant = m.from().add_offset(self.color_to_move().forward_offset());
            } else if !m.promotion() && m.capture() && m.special1() {
                // en passant capture: remove the pawn behind 'to'
                let sq = m.to().add_offset(-self.color_to_move().forward_offset());
                self.pieces[Piece::Pawn.idx()] = self.pieces[Piece::Pawn.idx()].clear_bit(sq.0);
            }
        }

        // Update castling rights
        if moved_piece == Piece::King {
            self.castling_rights[self.color_to_move().idx()] = [false, false];
        } else if moved_piece == Piece::Rook {
            let setups = &CASTLING_SETUPS[self.color_to_move().idx()];
            for setup in setups {
                if setup.rook_from == m.from() {
                    self.castling_rights[self.color_to_move().idx()][setup.castle_side.idx()] =
                        false;
                }
            }
        }
        if let Some(cp) = captured_piece
            && cp == Piece::Rook
        {
            let opp = self.color_to_move().opposite();
            for setup in &CASTLING_SETUPS[opp.idx()] {
                if setup.rook_from == m.to() {
                    self.castling_rights[opp.idx()][setup.castle_side.idx()] = false;
                }
            }
        }

        // Color bitboard: only destination matters (source becomes empty)
        self.white = self.white.set(to, self.white_to_move);
        self.white_to_move = !self.white_to_move;

        UnmakeInfo {
            prev_en_passant,
            prev_castling_rights,
            captured_piece,
            moved_piece,
        }
    }

    // Note: Board legality is expected to have been ensured before calling unmake.
    pub fn unmake_move(&mut self, m: Move, info: UnmakeInfo) {
        // The move we undo was made by the opposite color
        self.white_to_move = !self.white_to_move;
        let mover_color = self.color_to_move();
        let opponent_color = mover_color.opposite();

        let from = m.from().0;
        let to = m.to().0;
        let mask = (1u64 << from) | (1u64 << to);

        // Undo by type
        if !m.promotion() && !m.capture() && m.special0() {
            // Castle: move king and rook back
            let side: CastleSide = if m.special1() { QueenSide } else { KingSide };
            let setup = &CASTLING_SETUPS[mover_color.idx()][side as usize];
            self.pieces[Piece::King.idx()].0 ^= mask;
            self.pieces[Piece::Rook.idx()].0 ^=
                (1u64 << setup.rook_from.0) | (1u64 << setup.rook_to.0);
            // Update color bits for the occupied squares
            self.white = self
                .white
                .set(to, false)
                .set(from, mover_color == Color::White)
                .set(setup.rook_to.0, false)
                .set(setup.rook_from.0, mover_color == Color::White);
        } else if m.promotion() {
            // Remove promoted piece at 'to', restore pawn at 'from'
            self.pieces[m.promotion_piece().idx()] =
                self.pieces[m.promotion_piece().idx()].clear_bit(to);
            self.pieces[Piece::Pawn.idx()] = self.pieces[Piece::Pawn.idx()].set_bit(from);
            self.white = self
                .white
                .set(to, false)
                .set(from, mover_color == Color::White);
            if let Some(cp) = info.captured_piece {
                self.pieces[cp.idx()] = self.pieces[cp.idx()].set_bit(to);
                self.white = self.white.set(to, opponent_color == Color::White);
            }
        } else if m.capture() && m.special1() && info.moved_piece == Piece::Pawn {
            // En passant capture: move pawn back and restore captured pawn behind 'to'
            self.pieces[Piece::Pawn.idx()].0 ^= mask;
            self.white = self
                .white
                .set(to, false)
                .set(from, mover_color == Color::White);
            let cap_sq = Square(to).add_offset(-mover_color.forward_offset());
            self.pieces[Piece::Pawn.idx()] = self.pieces[Piece::Pawn.idx()].set_bit(cap_sq.0);
            self.white = self.white.set(cap_sq.0, opponent_color == Color::White);
        } else {
            // Quiet or normal capture
            self.pieces[info.moved_piece.idx()].0 ^= mask;
            self.white = self
                .white
                .set(to, false)
                .set(from, mover_color == Color::White);
            if let Some(cp) = info.captured_piece {
                self.pieces[cp.idx()] = self.pieces[cp.idx()].set_bit(to);
                self.white = self.white.set(to, opponent_color == Color::White);
            }
        }

        // Restore irreversible state
        self.en_passant = info.prev_en_passant;
        self.castling_rights = info.prev_castling_rights;
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

    /// Public accessor for piece lookup (used by move ordering)
    #[inline]
    pub fn piece_at_square(&self, square: Square) -> Option<(Piece, Color)> {
        self.piece_at(square.0)
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
        if rights[Color::White.idx()][0]
            && !(white_kings.is_set(Square::E1.0) && white_rooks.is_set(Square::H1.0))
        {
            rights[Color::White.idx()][0] = false;
        }
        // White queenside: King on E1 and rook on A1
        if rights[Color::White.idx()][1]
            && !(white_kings.is_set(Square::E1.0) && white_rooks.is_set(Square::A1.0))
        {
            rights[Color::White.idx()][1] = false;
        }
        // Black kingside: King on E8 and rook on H8
        if rights[Color::Black.idx()][0]
            && !(black_kings.is_set(Square::E8.0) && black_rooks.is_set(Square::H8.0))
        {
            rights[Color::Black.idx()][0] = false;
        }
        // Black queenside: King on E8 and rook on A8
        if rights[Color::Black.idx()][1]
            && !(black_kings.is_set(Square::E8.0) && black_rooks.is_set(Square::A8.0))
        {
            rights[Color::Black.idx()][1] = false;
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

    #[test]
    fn unmake_quiet_move_roundtrip() {
        let mut board = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        let original = board;
        let m = Move::new_quiet(Square::E2, Square::E4);
        let info = board.make_move_with_info(m).unwrap();
        assert!(board.pieces(Piece::Pawn, Color::White).is_set(Square::E4.0));
        board.unmake_move(m, info);
        assert_eq!(original.to_fen(), board.to_fen());
    }

    #[test]
    fn unmake_quiet_move_roundtrip_black_to_move() {
        let mut board = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0 1");
        let original = board;
        let m = Move::new_quiet(Square::E7, Square::E5);
        let info = board.make_move_with_info(m).unwrap();
        assert!(board.pieces(Piece::Pawn, Color::Black).is_set(Square::E5.0));
        board.unmake_move(m, info);
        assert_eq!(original.to_fen(), board.to_fen());
    }

    #[test]
    fn unmake_capture_roundtrip() {
        // Simple position where white captures a piece on e5
        let mut board =
            Board::from_fen("rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2");
        let original = board;
        let m = Move::new_capture(Square::E4, Square::D5);
        let info = board.make_move_with_info(m).unwrap();
        assert!(board.pieces(Piece::Pawn, Color::White).is_set(Square::D5.0));
        board.unmake_move(m, info);
        assert_eq!(original.to_fen(), board.to_fen());
    }

    #[test]
    fn unmake_capture_roundtrip_black_to_move() {
        // Simple position where white captures a piece on e5
        let mut board =
            Board::from_fen("rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 2");
        let original = board;
        let m = Move::new_capture(Square::D5, Square::E4);
        let info = board.make_move_with_info(m).unwrap();
        assert!(board.pieces(Piece::Pawn, Color::Black).is_set(Square::E4.0));
        board.unmake_move(m, info);
        assert_eq!(original.to_fen(), board.to_fen());
    }

    #[test]
    fn illegal_move_restores_board() {
        // Position where moving the pinned knight would expose king to check
        let mut board = Board::from_fen("4r3/8/8/8/8/8/4N3/4K3 w - - 0 1");
        let original_fen = board.to_fen();

        // Try to move the knight that would expose the king
        let illegal_move = Move::new_quiet(Square::E2, Square::C3);
        let result = board.make_move(illegal_move);

        // Move should fail
        assert!(result.is_err());

        // Board should be unchanged
        assert_eq!(board.to_fen(), original_fen);
    }
}
