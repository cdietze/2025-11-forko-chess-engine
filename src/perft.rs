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
        let nodes = foo(board, depth, depth);
        println!("Node count at depth {:?}: {}", depth, nodes);
        nodes
    }

    /// https://www.chessprogramming.org/Perft_Results
    #[test]
    fn test_perft_initial_position_depth_4() {
        let board = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        assert_eq!(perft(&board, 4), 197_281);
    }

    #[test]
    fn test_perft_position_2_depth_2() {
        let board =
            Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - ");
        assert_eq!(perft(&board, 2), 2039);
    }

    #[test]
    fn test_perft_position_2_depth_3() {
        let board =
            Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - ");
        assert_eq!(perft(&board, 3), 97862);
    }

    #[test]
    /// it takes too long, about 10 seconds
    #[ignore]
    fn test_perft_position_2_depth_4() {
        let board =
            Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - ");
        assert_eq!(perft(&board, 4), 4085603);
    }
}
