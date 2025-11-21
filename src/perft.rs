mod tests {
    use crate::board::Board;
    use crate::move_gen::generate_moves;

    fn perft(board: &Board, depth: u8) -> u64 {
        fn foo(b: &Board, d: u8, initial_depth: u8) -> u64 {
            let Ok(moves) = generate_moves(b) else {
                return 0;
            };
            if d == 0 {
                return 1;
            }
            let mut nodes = 0u64;
            for m in moves {
                // TODO: Don't clone board but use unmake_move
                let mut bb = *b;
                bb.make_move(m);
                let n = foo(&bb, d - 1, initial_depth);
                if d == initial_depth {
                    println!("{:?}: {}", m, n);
                }
                nodes += n;
            }
            nodes
        }
        foo(board, depth, depth)
    }

    /// https://www.chessprogramming.org/Perft_Results
    #[test]
    fn test_perft_initial_position_depth_4() {
        println!("Running perft tests...");

        let board = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        let perft_nodes = perft(&board, 4);
        println!("Perft nodes at depth 4: {}", perft_nodes);
        assert_eq!(perft_nodes, 197_281);
    }
}
