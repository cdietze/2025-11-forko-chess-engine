use crate::board::Board;
use crate::eval::eval;
use crate::r#move::Move;
use crate::move_gen::{generate_king_attack_map, generate_moves};

const CHECKMATE_SCORE: i32 = -1000000;
const STALEMATE_SCORE: i32 = 0;

#[derive(Debug)]
struct SearchInfo {
    node_count: u64,
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
        let opponent_attack_board =
            generate_king_attack_map(board, board.color_to_move().opposite());
        if opponent_attack_board
            .and(board.kings().and(board.own_color_board()))
            .is_not_empty()
        {
            return (CHECKMATE_SCORE, None);
        } else {
            return (STALEMATE_SCORE, None);
        }
    }
    for m in moves {
        let mut b = *board;
        b.make_move(m);
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
    use crate::search::CHECKMATE_SCORE;

    #[test]
    fn should_find_mate_in_1() {
        let mut board = Board::empty()
            .set_piece("e6".parse().unwrap(), Piece::King, Color::White)
            .set_piece("a6".parse().unwrap(), Piece::Rook, Color::White)
            .set_piece("e8".parse().unwrap(), Piece::King, Color::Black);
        let result = super::find_best_move(&mut board, 2);
        assert_eq!(result.move_.unwrap().algebraic(), "a6a8");
    }

    #[test]
    fn should_find_mate_in_2() {
        let mut board = Board::empty()
            .set_piece("b6".parse().unwrap(), Piece::King, Color::White)
            .set_piece("b1".parse().unwrap(), Piece::Rook, Color::White)
            .set_piece("a8".parse().unwrap(), Piece::King, Color::Black);
        let result = super::find_best_move(&mut board, 4);
        assert_eq!(result.score, -CHECKMATE_SCORE);
    }
}
