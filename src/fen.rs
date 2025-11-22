use crate::board::{Board, Color, Piece};
use crate::square::Square;

/// https://www.chessprogramming.org/Forsyth-Edwards_Notation
/// Very small, permissive parser sufficient for our tests:
/// - Parses piece placement and side to move
/// - Ignores castling/en passant/halfmove/fullmove fields
impl Board {
    pub fn from_fen(fen_string: &str) -> Board {
        let mut parts = fen_string.split_whitespace();
        let placement = parts.next().expect("FEN: missing placement");
        let active = parts.next().unwrap_or("w");

        // Start with an empty board
        let mut board = Board::empty();

        // Parse piece placement: ranks 8..1 from left to right (files a..h)
        let ranks: Vec<&str> = placement.split('/').collect();
        assert_eq!(ranks.len(), 8, "FEN: expected 8 ranks");

        for (i, rank_str) in ranks.iter().enumerate() {
            let rank_index = 7 - i as i32; // internal rank 0 is '1', FEN starts with rank '8'
            let mut file: u8 = 0;
            for ch in rank_str.chars() {
                if ch.is_ascii_digit() {
                    // Skip that many empty squares
                    file += ch.to_digit(10).unwrap() as u8;
                    continue;
                }
                // Determine piece and color
                let (piece, color) = match ch {
                    'K' => (Piece::King, Color::White),
                    'Q' => (Piece::Queen, Color::White),
                    'R' => (Piece::Rook, Color::White),
                    'B' => (Piece::Bishop, Color::White),
                    'N' => (Piece::Knight, Color::White),
                    'P' => (Piece::Pawn, Color::White),
                    'k' => (Piece::King, Color::Black),
                    'q' => (Piece::Queen, Color::Black),
                    'r' => (Piece::Rook, Color::Black),
                    'b' => (Piece::Bishop, Color::Black),
                    'n' => (Piece::Knight, Color::Black),
                    'p' => (Piece::Pawn, Color::Black),
                    _ => panic!("FEN: invalid piece char: {}", ch),
                };
                assert!(file < 8, "FEN: too many files in rank");
                let idx = (rank_index as u8) * 8 + file;
                board = board.set_piece(Square(idx), piece, color);
                file += 1;
            }
            assert_eq!(file, 8, "FEN: rank does not have 8 files");
        }

        // Active color
        board = match active {
            "w" | "W" => board.set_color_to_move(Color::White),
            "b" | "B" => board.set_color_to_move(Color::Black),
            _ => board, // keep default if weird; tests only use valid values
        };

        // Parse castling rights
        let castling = parts.next().unwrap_or("KQkq");
        let mut rights = [[false; 2]; 2];
        if castling != "-" {
            for ch in castling.chars() {
                match ch {
                    'K' => rights[Color::White.idx()][0] = true, // White kingside
                    'Q' => rights[Color::White.idx()][1] = true, // White queenside
                    'k' => rights[Color::Black.idx()][0] = true, // Black kingside
                    'q' => rights[Color::Black.idx()][1] = true, // Black queenside
                    _ => { /* ignore unexpected tokens to stay permissive */ }
                }
            }
        }
        board.castling_rights = rights;

        // Parse en passant square
        let ep_field = parts.next().unwrap_or("-");
        board.en_passant = match ep_field {
            "-" => Square::ILLEGAL_SQUARE,
            s => s.parse::<Square>().unwrap_or(Square::ILLEGAL_SQUARE),
        };

        board.normalize();
        board
    }

    /// Convert the current board position to a FEN string.
    ///
    /// Notes:
    /// - We emit piece placement and active color.
    /// - Castling availability, en passant square, halfmove clock and fullmove number
    ///   are not tracked in this engine yet, so we output "- - 0 1" for those fields.
    pub fn to_fen(&self) -> String {
        // Build piece placement from rank 8 to 1, files a to h
        let mut ranks: Vec<String> = Vec::with_capacity(8);
        for rank in (0..8).rev() {
            let mut rank_str = String::new();
            let mut empty_run = 0u8;
            for file in 0..8u8 {
                let idx = (rank * 8 + file as i32) as u8;
                // Is the square occupied?
                let occupied = self.occupied().is_set(idx);
                if !occupied {
                    empty_run += 1;
                    continue;
                }
                // Flush any pending empties
                if empty_run > 0 {
                    rank_str.push(char::from(b'0' + empty_run));
                    empty_run = 0;
                }

                // Determine color
                let is_white = self.white.is_set(idx);
                // Determine piece by scanning piece bitboards
                let mut ch = '?';
                for i in 0..Piece::COUNT {
                    if self.pieces[i].is_set(idx) {
                        ch = match i {
                            0 => 'k',
                            1 => 'q',
                            2 => 'r',
                            3 => 'b',
                            4 => 'n',
                            5 => 'p',
                            _ => unreachable!(),
                        };
                        break;
                    }
                }
                if is_white {
                    ch = ch.to_ascii_uppercase();
                }
                rank_str.push(ch);
            }
            if empty_run > 0 {
                rank_str.push(char::from(b'0' + empty_run));
            }
            ranks.push(rank_str);
        }

        let placement = ranks.join("/");
        let active = if self.white_to_move { "w" } else { "b" };
        format!("{} {} - - 0 1", placement, active)
    }
}

mod tests {
    use super::*;
    use crate::bitboard::BitBoard;
    #[test]
    fn test_parse_fen() {
        let board = Board::from_fen("k7/8/8/8/8/8/8/7K w - - 0 1");
        println!("{:?}", board);
        assert_eq!(board.white_to_move, true);
        assert_eq!(
            board.white_kings(),
            BitBoard::try_from_coords(["h1"]).unwrap()
        );
    }

    #[test]
    fn test_to_fen_kings_only() {
        let fen = "k7/8/8/8/8/8/8/7K w - - 0 1";
        let board = Board::from_fen(fen);
        assert_eq!(board.to_fen(), fen);
    }

    #[test]
    fn test_to_fen_black_to_move() {
        let fen = "k7/8/8/8/8/8/8/7K b - - 0 1";
        let board = Board::from_fen(fen);
        assert_eq!(board.to_fen(), fen);
    }
}
