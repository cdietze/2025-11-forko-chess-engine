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

        board
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
}
