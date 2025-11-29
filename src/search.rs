use crate::board::Board;
use crate::board::Color::White;
use crate::eval::eval;
use crate::r#move::Move;
use crate::move_gen::{generate_moves, king_attack_map};
use crate::transposition::{NodeType, TTEntry, TranspositionTable, position_key};
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

pub fn find_best_move(board: &mut Board, depth: u8, tt: &mut TranspositionTable) -> SearchResult {
    let mut info = SearchInfo { node_count: 0 };
    let mut best_overall_move: Option<Move> = None;
    let mut best_overall_score: i32 = -INF + 1;

    for d in 1..=depth {
        let (score, m) = nega_max_impl(
            board,
            d,
            -INF + 1,
            INF - 1,
            &mut info,
            true,
            tt,
            best_overall_move,
        );
        if m.is_some() {
            best_overall_move = m;
        }
        best_overall_score = score;
    }
    println!(
        "find_best_move done: score: {}, move: {:?}, {:?}, transposition table size: {:?}",
        best_overall_score,
        best_overall_move,
        info,
        with_separator(tt.map.len() as i32)
    );
    SearchResult {
        score: best_overall_score,
        move_: best_overall_move,
    }
}

fn nega_max_impl(
    board: &mut Board,
    depth: u8,
    mut alpha: i32,
    beta: i32,
    info: &mut SearchInfo,
    track_move: bool,
    tt: &mut TranspositionTable,
    preferred_move: Option<Move>,
) -> (i32, Option<Move>) {
    info.node_count += 1;
    let key = position_key(board);
    let orig_alpha = alpha;
    let orig_beta = beta;

    // TT probe
    if let Some(entry) = tt.get(key)
        && entry.depth >= depth
    {
        match entry.flag {
            NodeType::Exact => {
                return (entry.score, if track_move { entry.best_move } else { None });
            }
            NodeType::LowerBound => {
                if entry.score >= beta {
                    return (entry.score, if track_move { entry.best_move } else { None });
                }
                if entry.score > alpha {
                    alpha = entry.score;
                }
            }
            NodeType::UpperBound => {
                if entry.score <= alpha {
                    return (entry.score, if track_move { entry.best_move } else { None });
                }
            }
        }
    }
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
    let mut moves = generate_moves(board);
    // Try TT move first if present
    if let Some(entry) = tt.get(key)
        && let Some(tt_move) = entry.best_move
        && let Some(idx) = moves.iter().position(|m| *m == tt_move)
    {
        let m = moves.remove(idx);
        moves.insert(0, m);
    }
    // Then PV move from previous iteration
    if let Some(pv) = preferred_move
        && let Some(idx) = moves.iter().position(|m| *m == pv)
    {
        let m = moves.remove(idx);
        moves.insert(0, m);
    }
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
        let score = -nega_max_impl(&mut b, depth - 1, -beta, -alpha, info, false, tt, None).0;
        if score > best_score {
            best_score = score;
            // Always track the local best move for TT storage; return value will honor track_move
            best_move = Some(m);
        }
        // alpha-beta pruning
        if best_score > alpha {
            alpha = best_score;
        }
        if alpha >= beta {
            break;
        }
    }
    // Store in TT — use original alpha/beta to set the correct bound type
    let flag = if best_score <= orig_alpha {
        NodeType::UpperBound
    } else if best_score >= orig_beta {
        NodeType::LowerBound
    } else {
        NodeType::Exact
    };
    tt.store(TTEntry {
        key,
        depth,
        score: best_score,
        flag,
        best_move,
    });
    (best_score, best_move)
}

#[cfg(test)]
mod tests {
    use crate::board::Board;
    use crate::search;
    use crate::search::CHECKMATE_SCORE;
    use crate::transposition::TranspositionTable;
    use crate::util::with_separator;

    fn new_transposition_table() -> TranspositionTable {
        TranspositionTable::new(1_000_000)
    }

    #[test]
    fn should_find_mate_in_1() {
        let mut board = Board::from_fen("4k3/8/R3K3/8/8/8/8/8 w - - 0 1");
        let result = super::find_best_move(&mut board, 2, &mut new_transposition_table());
        assert_eq!(result.move_.unwrap().algebraic(), "a6a8");
    }

    #[test]
    fn should_find_mate_in_2() {
        let mut board = Board::from_fen("k7/8/1K6/8/8/8/8/1R6 w - - 0 1");
        let result = super::find_best_move(&mut board, 4, &mut new_transposition_table());
        assert!(result.score >= -CHECKMATE_SCORE);
    }

    #[test]
    fn should_find_mate_in_2_for_black() {
        let mut board = Board::from_fen("K7/8/1k6/8/8/8/8/1r6 b - - 0 1");
        let result = super::find_best_move(&mut board, 4, &mut new_transposition_table());
        assert!(result.score >= -CHECKMATE_SCORE);
    }

    #[test]
    fn should_find_single_best_move_for_white() {
        let mut board = Board::from_fen("k1q5/8/8/3N4/8/8/8/K3R3 w - - 0 1");
        let result = super::find_best_move(&mut board, 6, &mut new_transposition_table());
        println!("result: {:?}", result.move_);
        assert_eq!(result.move_.unwrap().algebraic(), "d5b6");
    }

    #[test]
    fn should_find_single_best_move_for_black() {
        let mut board = Board::from_fen("K1Q5/8/8/3n4/8/8/8/k3r3 b - - 0 1");
        let result = super::find_best_move(&mut board, 6, &mut new_transposition_table());
        println!("result: {:?}", result.move_);
        assert_eq!(result.move_.unwrap().algebraic(), "d5b6");
    }

    #[test]
    fn case_1() {
        let mut board = Board::from_fen("2bqkb2/4pp2/8/1B6/8/5N2/5PPP/5QK1 b - - 0 1");
        let result = super::find_best_move(&mut board, 2, &mut new_transposition_table());
        println!("result: {:?}", result.move_);
        assert_eq!(result.move_.unwrap().algebraic(), "c8d7");
    }

    #[test]
    #[ignore]
    fn should_play_against_self() {
        let mut board = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        let mut tt = TranspositionTable::new(1_000_000);

        println!(
            "Starting position ({} to move):\n{}",
            board.color_to_move(),
            board
        );

        for ply in 1..=50 {
            let result = search::find_best_move(&mut board, 6, &mut tt);
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
