#[cfg(test)]
mod tests {
    use crate::board::Board;
    use crate::move_gen::generate_moves;

    fn perft(board: &Board, depth: u8) -> u64 {
        fn foo(b: &Board, d: u8, initial_depth: u8) -> u64 {
            if d == 0 {
                return 1;
            }
            let moves = generate_moves(b);
            let mut nodes = 0u64;
            for m in moves {
                // TODO: Don't clone board but use unmake_move
                let mut bb = *b;
                if bb.make_move(m).is_err() {
                    continue;
                }
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
        assert_eq!(perft(&board, 2), 2_039);
    }

    #[test]
    fn test_perft_position_2_depth_3() {
        let board =
            Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - ");
        assert_eq!(perft(&board, 3), 97_862);
    }

    #[test]
    #[cfg_attr(debug_assertions, ignore)]
    fn test_perft_position_2_depth_4() {
        let board =
            Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - ");
        assert_eq!(perft(&board, 4), 4_085_603);
    }

    #[test]
    #[cfg_attr(debug_assertions, ignore)]
    fn test_perft_position_2_depth_5() {
        let board =
            Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - ");
        assert_eq!(perft(&board, 5), 193_690_690);
    }
    #[test]
    #[cfg_attr(debug_assertions, ignore)]
    fn test_perft_position_3_depth_6() {
        let board = Board::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1");
        assert_eq!(perft(&board, 6), 11_030_083);
    }

    #[test]
    #[cfg_attr(debug_assertions, ignore)]
    fn test_perft_position_3_depth_7() {
        let board = Board::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1");
        assert_eq!(perft(&board, 7), 178_633_661);
    }

    #[test]
    #[cfg_attr(debug_assertions, ignore)]
    fn test_perft_position_4_depth_5() {
        let board =
            Board::from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1");
        assert_eq!(perft(&board, 5), 15_833_292);
    }
    #[test]
    #[cfg_attr(debug_assertions, ignore)]
    fn test_perft_position_5_depth_5() {
        let board = Board::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8");
        assert_eq!(perft(&board, 5), 89_941_194);
    }
}
