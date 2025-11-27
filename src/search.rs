use crate::board::Board;
use crate::board::Color::White;
use crate::eval::eval;
use crate::r#move::Move;
use crate::move_gen::{generate_moves, king_attack_map};
use crate::util::with_separator;

const INF: i32 = 1_000_000_000; // search bounds for alpha-beta
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
    let (best_score, best_move) = nega_max_impl(board, depth, -INF + 1, INF - 1, &mut info, true);
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
    mut alpha: i32,
    beta: i32,
    info: &mut SearchInfo,
    track_move: bool,
) -> (i32, Option<Move>) {
    info.node_count += 1;
    if depth == 0 {
        let e = eval(board);
        let e_stm = if board.color_to_move() == White {
            e
        } else {
            -e
        };
        return (e_stm, None);
    }
    let mut best_score = -INF + 1;
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
        let score = -nega_max_impl(&mut b, depth - 1, -beta, -alpha, info, false).0;
        if score > best_score {
            best_score = score;
            if track_move {
                best_move = Some(m);
            }
        }
        // alpha-beta pruning
        if best_score > alpha {
            alpha = best_score;
        }
        if alpha >= beta {
            break;
        }
    }
    (best_score, best_move)
}

#[cfg(test)]
mod tests {
    use crate::board::Board;
    use crate::search;
    use crate::search::CHECKMATE_SCORE;
    use crate::util::with_separator;

    #[test]
    fn should_find_mate_in_1() {
        let mut board = Board::from_fen("4k3/8/R3K3/8/8/8/8/8 w - - 0 1");
        let result = super::find_best_move(&mut board, 2);
        assert_eq!(result.move_.unwrap().algebraic(), "a6a8");
    }

    #[test]
    fn should_find_mate_in_2() {
        let mut board = Board::from_fen("k7/8/1K6/8/8/8/8/1R6 w - - 0 1");
        let result = super::find_best_move(&mut board, 4);
        assert!(result.score >= -CHECKMATE_SCORE);
    }

    #[test]
    fn should_find_mate_in_2_for_black() {
        let mut board = Board::from_fen("K7/8/1k6/8/8/8/8/1r6 b - - 0 1");
        let result = super::find_best_move(&mut board, 4);
        assert!(result.score >= -CHECKMATE_SCORE);
    }

    #[test]
    fn should_find_single_best_move_for_white() {
        let mut board = Board::from_fen("k1q5/8/8/3N4/8/8/8/K3R3 w - - 0 1");
        let result = super::find_best_move(&mut board, 6);
        println!("result: {:?}", result.move_);
        assert_eq!(result.move_.unwrap().algebraic(), "d5b6");
    }

    #[test]
    fn should_find_single_best_move_for_black() {
        let mut board = Board::from_fen("K1Q5/8/8/3n4/8/8/8/k3r3 b - - 0 1");
        let result = super::find_best_move(&mut board, 6);
        println!("result: {:?}", result.move_);
        assert_eq!(result.move_.unwrap().algebraic(), "d5b6");
    }

    #[test]
    fn case_1() {
        let mut board = Board::from_fen("2bqkb2/4pp2/8/1B6/8/5N2/5PPP/5QK1 b - - 0 1");
        let result = super::find_best_move(&mut board, 2);
        println!("result: {:?}", result.move_);
        assert_eq!(result.move_.unwrap().algebraic(), "c8d7");
    }

    #[test]
    #[ignore]
    fn should_play_against_self() {
        let mut board = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");

        println!(
            "Starting position ({} to move):\n{}",
            board.color_to_move(),
            board
        );

        for ply in 1..=50 {
            let result = search::find_best_move(&mut board, 4);
            match result.move_ {
                Some(m) => {
                    println!(
                        "Ply {ply}: best move = {} (score: {})",
                        m,
                        with_separator(result.score)
                    );
                    board.make_move(m).unwrap();
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
