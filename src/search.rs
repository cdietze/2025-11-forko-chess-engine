use crate::board::Board;
use crate::eval::eval;
use crate::r#move::Move;
use crate::move_gen::{MoveGenError, generate_moves, king_attack_map};
use crate::util::with_separator;

const ILLEGAL_POSITION_SCORE: i32 = -1_000_000_000;
const CHECKMATE_SCORE: i32 = -1_000_000;
const STALEMATE_SCORE: i32 = 0;

struct SearchInfo {
    node_count: u64,
}

impl std::fmt::Debug for SearchInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SearchInfo {{ node_count: {} }}",
            with_separator(self.node_count as i32)
        )
    }
}

pub struct SearchResult {
    pub score: i32,
    pub move_: Option<Move>,
}

pub fn find_best_move(board: &mut Board, depth: u8) -> SearchResult {
    let mut info = SearchInfo { node_count: 0 };
    let (best_score, best_move) = nega_max_impl(board, depth, &mut info, true);
    println!(
        "find_best_move done: score: {}, move: {:?}, {:?}",
        best_score, best_move, info
    );
    SearchResult {
        score: best_score,
        move_: best_move,
    }
}

fn nega_max_impl(
    board: &mut Board,
    depth: u8,
    info: &mut SearchInfo,
    track_move: bool,
) -> (i32, Option<Move>) {
    info.node_count += 1;
    if depth == 0 {
        return (eval(board), None);
    }
    let mut best_score = i32::MIN;
    let mut best_move = None;
    let moves = generate_moves(board);
    if moves.is_empty() {
        let opponent_attack_board = king_attack_map(board, board.color_to_move().opposite());
        if opponent_attack_board
            .and(board.kings().and(board.own_color_board()))
            .is_not_empty()
        {
            return (CHECKMATE_SCORE - depth as i32, None);
        } else {
            return (STALEMATE_SCORE, None);
        }
    }
    for m in moves {
        // TODO: Don't clone board but use unmake_move
        let mut b = *board;
        let r = b.make_move(m);
        if r.is_err() {
            continue;
        }
        let score = -nega_max_impl(&mut b, depth - 1, info, false).0;
        if score > best_score {
            best_score = score;
            if track_move {
                best_move = Some(m);
            }
        }
    }
    (best_score, best_move)
}

/// Evaluation relative to side to move.
// eval function now provided by crate::eval::eval

mod tests {
    use crate::board::{Board, Color, Piece};
    use crate::search;
    use crate::search::CHECKMATE_SCORE;
    use crate::util::with_separator;

    #[test]
    fn should_find_mate_in_1() {
        let mut board = Board::empty()
            .set_piece("e6".parse().unwrap(), Piece::King, Color::White)
            .set_piece("a6".parse().unwrap(), Piece::Rook, Color::White)
            .set_piece("e8".parse().unwrap(), Piece::King, Color::Black)
            .normalize();
        let result = super::find_best_move(&mut board, 2);
        assert_eq!(result.move_.unwrap().algebraic(), "a6a8");
    }

    #[test]
    fn should_find_mate_in_2() {
        let mut board = Board::empty()
            .set_piece("b6".parse().unwrap(), Piece::King, Color::White)
            .set_piece("b1".parse().unwrap(), Piece::Rook, Color::White)
            .set_piece("a8".parse().unwrap(), Piece::King, Color::Black)
            .normalize();
        let result = super::find_best_move(&mut board, 4);
        assert!(result.score >= -CHECKMATE_SCORE);
    }

    #[test]
    fn should_play_against_itself() {
        let mut board = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");

        println!(
            "Starting position ({} to move):\n{}",
            board.color_to_move(),
            board
        );

        for ply in 1..=100 {
            let result = search::find_best_move(&mut board, 2);
            match result.move_ {
                Some(m) => {
                    println!(
                        "Ply {ply}: best move = {} (score: {})",
                        m,
                        with_separator(result.score)
                    );
                    board.make_move(m);
                    println!(
                        "After ply {ply} ({} to move):\n{}\n{}",
                        board.color_to_move(),
                        board.to_fen(),
                        board
                    );
                }
                None => {
                    println!("Ply {ply}: no legal move found. Score: {}", result.score);
                    println!(
                        "Final position ({} to move):\n{}",
                        board.color_to_move(),
                        board
                    );
                    break;
                }
            }
        }
    }
}
