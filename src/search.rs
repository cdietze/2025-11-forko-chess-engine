use crate::board::Board;
use crate::board::Color::White;
use crate::eval::eval;
use crate::r#move::Move;
use crate::move_gen::{generate_moves, king_attack_map};
use crate::move_ordering::{KillerMoves, MovePicker};
use crate::transposition::{NodeType, TTEntry, TranspositionTable, position_key};
use crate::util::with_separator;

const INF: i32 = 1_000_000_000; // search bounds for alpha-beta
const CHECKMATE_SCORE: i32 = -1_000_000;
const STALEMATE_SCORE: i32 = 0;

struct SearchInfo {
    node_count: u64,
    qsearch_node_count: u64,
}

impl std::fmt::Debug for SearchInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SearchInfo {{ nodes: {}, qnodes: {} }}",
            with_separator(self.node_count as i32),
            with_separator(self.qsearch_node_count as i32)
        )
    }
}

pub struct SearchResult {
    pub score: i32,
    pub move_: Option<Move>,
    pub nodes: u64,
}

pub fn find_best_move(board: &mut Board, depth: u8, tt: &mut TranspositionTable) -> SearchResult {
    let mut info = SearchInfo {
        node_count: 0,
        qsearch_node_count: 0,
    };
    let mut killers = KillerMoves::new();
    let mut best_overall_move: Option<Move> = None;
    let mut best_overall_score: i32 = -INF + 1;

    for d in 1..=depth {
        killers.clear(); // Clear killers each iteration
        let (score, m) = nega_max_impl(
            board,
            d,
            -INF + 1,
            INF - 1,
            &mut info,
            true,
            tt,
            best_overall_move,
            &mut killers,
            0,
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
    let total_nodes = info.node_count + info.qsearch_node_count;
    SearchResult {
        score: best_overall_score,
        move_: best_overall_move,
        nodes: total_nodes,
    }
}

/// Quiescence search: extends search on tactical moves until position is quiet
fn quiescence(board: &mut Board, mut alpha: i32, beta: i32, info: &mut SearchInfo) -> i32 {
    info.qsearch_node_count += 1;

    // Stand-pat: static evaluation
    let eval_score = eval(board);
    let stand_pat = if board.color_to_move() == White {
        eval_score
    } else {
        -eval_score
    };

    if stand_pat >= beta {
        return beta;
    }
    if stand_pat > alpha {
        alpha = stand_pat;
    }

    // Generate and search only captures
    let moves = generate_moves(board);

    for m in moves {
        // Only search captures and promotions
        if !m.capture() && !m.promotion() {
            continue;
        }

        let mut b = *board;
        if b.make_move(m).is_err() {
            continue;
        }
        let score = -quiescence(&mut b, -beta, -alpha, info);

        if score >= beta {
            return beta;
        }
        if score > alpha {
            alpha = score;
        }
    }
    alpha
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
    killers: &mut KillerMoves,
    ply: usize,
) -> (i32, Option<Move>) {
    info.node_count += 1;
    let key = position_key(board);
    let orig_alpha = alpha;
    let orig_beta = beta;

    // TT probe
    let tt_move = if let Some(entry) = tt.get(key) {
        if entry.depth >= depth {
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
        entry.best_move
    } else {
        None
    };

    // At leaf nodes, enter quiescence search
    if depth == 0 {
        return (quiescence(board, alpha, beta, info), None);
    }

    let moves = generate_moves(board);
    if moves.is_empty() {
        let opponent_attack_board = king_attack_map(board, board.color_to_move().opposite());
        if opponent_attack_board
            .and(board.kings().and(board.own_color_board()))
            .is_not_empty()
        {
            return (CHECKMATE_SCORE + ply as i32, None);
        } else {
            return (STALEMATE_SCORE, None);
        }
    }

    let mut best_score = -INF + 1;
    let mut best_move = None;

    // Determine which move to try first (TT or PV)
    let first_move = tt_move.or(preferred_move);
    let killer_moves = killers.get(ply);

    // Use MovePicker for efficient ordering
    let mut picker = MovePicker::new(board, moves, first_move, killer_moves);

    while let Some(m) = picker.next() {
        let mut b = *board;
        if b.make_move(m).is_err() {
            continue;
        }

        let score = -nega_max_impl(
            &mut b,
            depth - 1,
            -beta,
            -alpha,
            info,
            false,
            tt,
            None,
            killers,
            ply + 1,
        )
        .0;

        if score > best_score {
            best_score = score;
            best_move = Some(m);
        }

        if best_score > alpha {
            alpha = best_score;
        }

        if alpha >= beta {
            // Beta cutoff: store killer move if it's quiet
            if !m.capture() && !m.promotion() {
                killers.store(ply, m);
            }
            break;
        }
    }

    // Store in TT
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

    (best_score, if track_move { best_move } else { None })
}

#[cfg(test)]
mod tests {
    use crate::board::Board;
    use crate::fen::STARTPOS_FEN;
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
        let ply = 3;
        assert!(result.score == -CHECKMATE_SCORE - ply);
    }

    #[test]
    fn should_find_mate_in_2_for_black() {
        let mut board = Board::from_fen("K7/8/1k6/8/8/8/8/1r6 b - - 0 1");
        let result = super::find_best_move(&mut board, 4, &mut new_transposition_table());
        let ply = 3;
        assert!(result.score >= -CHECKMATE_SCORE - ply);
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
    #[cfg_attr(debug_assertions, ignore)]
    fn search_statistics_depth_run() {
        use std::time::Instant;
        let mut board = Board::from_fen(STARTPOS_FEN);
        let mut tt = TranspositionTable::new(1_000_000);
        let depth = 10;
        let start = Instant::now();
        let result = super::find_best_move(&mut board, depth, &mut tt);
        let elapsed = start.elapsed();
        println!(
            "Search stats -> depth: {}, score: {}, best: {}, elapsed: {:?}, tt_size: {}",
            depth,
            with_separator(result.score),
            result.move_.unwrap().algebraic(),
            elapsed,
            with_separator(tt.map.len() as i32)
        );
        let secs = elapsed.as_secs_f64().max(1e-9);
        let nps = with_separator(((result.nodes as f64) / secs) as i32);
        println!(
            "Node count at depth {:?}: {} | time: {:.3}s | nps: {}",
            depth, result.nodes, secs, nps
        );
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
            let result = search::find_best_move(&mut board, 8, &mut tt);
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
