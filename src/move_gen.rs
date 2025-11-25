use crate::bitboard::BitBoard;
use crate::board::Color::White;
use crate::board::{Board, CastlingRights, Color, Piece};
use crate::geometry::{Dir8, between_bb, line_bb};
use crate::r#move::Move;
use crate::precomputed::{CASTLING_SETUPS, CastleSide, king_moves, knight_attacks, ray_attacks};
use crate::square::Square;

const PROMOTION_PIECES: [Piece; 4] = [Piece::Queen, Piece::Rook, Piece::Bishop, Piece::Knight];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveGenError {
    IllegalPosition,
}

/// Generates a list of *pseudo-legal* moves from given board.
pub fn generate_moves(board: &Board) -> Vec<Move> {
    let mut v = Vec::new();

    let own_color: BitBoard = if board.white_to_move {
        board.white
    } else {
        board.white.not()
    };
    let occupied = board.occupied();
    let opp_pieces: BitBoard = occupied.and(own_color.not());
    let own_king = Square(board.kings().and(own_color).bit_scan_forward());
    if !own_king.is_valid() {
        panic!("Invalid own king square: {:?}", own_king);
    }
    assert!(own_king.is_valid());
    let opp_rq = board.pieces[Piece::Rook.idx()]
        .or(board.pieces[Piece::Queen.idx()])
        .and(opp_pieces);
    let opp_bq = board.pieces[Piece::Bishop.idx()]
        .or(board.pieces[Piece::Queen.idx()])
        .and(opp_pieces);
    let pinned = pinned(PinnedProps {
        king_square: own_king,
        occupied,
        own_pieces: own_color,
        opp_rq,
        opp_bq,
    });
    let king_attack_map = king_attack_map(board, board.color_to_move().opposite());

    let attacks_to_king = attacks_to_king(AttacksToKingProps {
        king_square: own_king,
        color_to_move: board.color_to_move(),
        occupied,
        opp_color_board: own_color.not(),
        pieces: board.pieces,
    });

    let num_checks = attacks_to_king.0.count_ones();

    // println!("board:\n{}", board);
    // println!("num_checks: {}", num_checks);
    // println!("king_attack_map: {:?}", king_attack_map);
    // println!("attacks_to_king: {:?}", attacks_to_king);
    // println!("pinned: {:?}", pinned);

    if num_checks == 0 {
        let not_own_pieces_bb = occupied.and(board.own_color_board()).not();
        add_castling_moves(
            AddCastlingMovesProps {
                king_square: own_king,
                side_to_move: board.color_to_move(),
                occupied,
                king_attack_map,
                castling_rights: board.castling_rights[board.color_to_move().idx()],
            },
            &mut v,
        );
        add_king_moves(
            AddKingMovesProps {
                king_square: own_king,
                occupied,
                to_mask: not_own_pieces_bb.and(king_attack_map.not()),
            },
            &mut v,
        );
        add_piece_moves(
            AddPieceMovesProps {
                board: &board,
                own_king,
                occupied,
                pinned,
                to_mask: not_own_pieces_bb,
                opp_pieces,
            },
            &mut v,
        );
    } else if num_checks == 1 {
        let not_own_pieces_bb = occupied.and(board.own_color_board()).not();
        let checking_piece = attacks_to_king.bit_scan_forward();
        let attack_line_bb = between_bb(own_king, Square(checking_piece));
        // King moves to safe square
        add_king_moves(
            AddKingMovesProps {
                king_square: own_king,
                occupied,
                to_mask: not_own_pieces_bb.and(king_attack_map.not()),
            },
            &mut v,
        );
        // To lift the check, the only possible moves are to capture the checking piece or block the checking piece
        let lift_check_mask = attacks_to_king.or(attack_line_bb);
        add_piece_moves(
            AddPieceMovesProps {
                board: &board,
                own_king,
                occupied,
                pinned,
                to_mask: not_own_pieces_bb.and(lift_check_mask),
                opp_pieces,
            },
            &mut v,
        );
    } else if num_checks > 1 {
        let not_own_pieces_bb = occupied.and(board.own_color_board()).not();
        // In double check, only king moves to safe squares are possible
        add_king_moves(
            AddKingMovesProps {
                king_square: own_king,
                occupied,
                to_mask: not_own_pieces_bb.and(king_attack_map.not()),
            },
            &mut v,
        );
    }
    v
}

pub fn is_legal(board: &Board) -> bool {
    let own_color: BitBoard = if board.white_to_move {
        board.white
    } else {
        board.white.not()
    };
    let opp_color: BitBoard = own_color.not();
    let occupied = board.occupied();
    let opp_king = Square(board.kings().and(opp_color).bit_scan_forward());
    let attacks_to_opp_king = attacks_to_king(AttacksToKingProps {
        king_square: opp_king,
        color_to_move: board.color_to_move().opposite(),
        occupied,
        opp_color_board: own_color,
        pieces: board.pieces,
    });
    if attacks_to_opp_king.is_not_empty() {
        // Opponent king is currently in check in the given position, which should
        // not happen for a legal game position after a legal move.
        return false;
    }
    true
}

struct AddPieceMovesProps<'a> {
    board: &'a Board,
    own_king: Square,
    occupied: BitBoard,
    pinned: BitBoard,
    to_mask: BitBoard,
    opp_pieces: BitBoard,
}

/// Adds any piece moves except for kings.
fn add_piece_moves(props: AddPieceMovesProps, v: &mut Vec<Move>) {
    let board = props.board;
    add_sliding_pieces_moves(
        AddSlidingPiecesMovesProps {
            queens: board.pieces(Piece::Queen, board.color_to_move()),
            rooks: board.pieces(Piece::Rook, board.color_to_move()),
            bishops: board.pieces(Piece::Bishop, board.color_to_move()),
            king_square: props.own_king,
            occupied: props.occupied,
            pinned: props.pinned,
            to_mask: props.to_mask,
        },
        v,
    );
    add_knight_moves(
        AddKnightMovesProps {
            knights: board.pieces(Piece::Knight, board.color_to_move()),
            occupied: props.occupied,
            to_mask: props.to_mask,
        },
        v,
    );
    add_pawn_moves(
        AddPawnMovesProps {
            pawns: board.pieces(Piece::Pawn, board.color_to_move()),
            color_to_move: board.color_to_move(),
            to_mask: props.to_mask,
            not_occupied: props.occupied.not(),
            attack_targets: props.opp_pieces,
            en_passant: board.en_passant,
        },
        v,
    );
}

struct AddKingMovesProps {
    king_square: Square,
    occupied: BitBoard,
    to_mask: BitBoard,
}

fn add_king_moves(props: AddKingMovesProps, v: &mut Vec<Move>) {
    let tos = king_moves(props.king_square);
    let tos = tos.and(props.to_mask);
    tos.and(props.occupied).for_each_set_bit(|to_square| {
        v.push(Move::new_capture(props.king_square, to_square));
        true
    });
    tos.and(props.occupied.not()).for_each_set_bit(|to_square| {
        v.push(Move::new_quiet(props.king_square, to_square));
        true
    });
}

struct AddCastlingMovesProps {
    king_square: Square,
    side_to_move: Color,
    occupied: BitBoard,
    king_attack_map: BitBoard,
    castling_rights: CastlingRights,
}
fn add_castling_moves(props: AddCastlingMovesProps, v: &mut Vec<Move>) {
    let setups = &CASTLING_SETUPS[props.side_to_move.idx()];

    for side in CastleSide::ALL {
        if !props.castling_rights[side.idx()] {
            continue;
        }
        let setup = &setups[side.idx()];
        if setup.safe_squares.and(props.king_attack_map).is_not_empty() {
            continue;
        }
        if setup.empty_squares.and(props.occupied).is_not_empty() {
            continue;
        }
        v.push(Move::new_castle(setup.king_from, setup.king_to, side));
    }
}

struct AddSlidingPiecesMovesProps {
    queens: BitBoard,
    rooks: BitBoard,
    bishops: BitBoard,
    king_square: Square,
    occupied: BitBoard,
    pinned: BitBoard,
    to_mask: BitBoard,
}
fn add_sliding_pieces_moves(props: AddSlidingPiecesMovesProps, v: &mut Vec<Move>) {
    let add_sliding_moves = |square: Square, tos: BitBoard, v: &mut Vec<Move>| {
        let mut tos = tos.and(props.to_mask);
        if props.pinned.has_square(square) {
            tos = tos.and(line_bb(props.king_square, square))
        }
        tos.and(props.occupied).for_each_set_bit(|to_square| {
            v.push(Move::new_capture(square, to_square));
            true
        });
        tos.and(props.occupied.not()).for_each_set_bit(|to_square| {
            v.push(Move::new_quiet(square, to_square));
            true
        });
    };
    props.queens.for_each_set_bit(|square| {
        add_sliding_moves(square, queen_attacks(square, props.occupied), v);
        true
    });
    props.bishops.for_each_set_bit(|square| {
        add_sliding_moves(square, bishop_attacks(square, props.occupied), v);
        true
    });
    props.rooks.for_each_set_bit(|square| {
        add_sliding_moves(square, rook_attacks(square, props.occupied), v);
        true
    });
}
struct AddKnightMovesProps {
    knights: BitBoard,
    occupied: BitBoard,
    to_mask: BitBoard,
}
fn add_knight_moves(props: AddKnightMovesProps, v: &mut Vec<Move>) {
    props.knights.for_each_set_bit(|knight_square| {
        let tos = knight_attacks(knight_square);
        let tos = tos.and(props.to_mask);
        tos.and(props.occupied).for_each_set_bit(|to_square| {
            v.push(Move::new_capture(knight_square, to_square));
            true
        });
        tos.and(props.occupied.not()).for_each_set_bit(|to_square| {
            v.push(Move::new_quiet(knight_square, to_square));
            true
        });
        true
    });
}

struct AddPawnMovesProps {
    pawns: BitBoard,
    color_to_move: Color,
    to_mask: BitBoard,
    not_occupied: BitBoard,
    attack_targets: BitBoard,
    en_passant: Square,
}

fn add_pawn_moves(props: AddPawnMovesProps, v: &mut Vec<Move>) {
    let promotion_rank = match props.color_to_move {
        White => BitBoard::RANK_8,
        _ => BitBoard::RANK_1,
    };
    let not_promotion_rank = promotion_rank.not();
    // Single pushes
    let forward_offset = props.color_to_move.forward_offset();
    let single_push = pawn_single_push(props.pawns, props.not_occupied, props.color_to_move);
    let mut tos = single_push.and(props.to_mask);
    tos.and(promotion_rank).for_each_set_bit(|to_square| {
        let from = Square((to_square.0 as i8 - forward_offset) as u8);
        PROMOTION_PIECES.iter().for_each(|p| {
            v.push(Move::new_promotion(from, to_square, false, *p));
        });
        true
    });
    tos.and(not_promotion_rank).for_each_set_bit(|to_square| {
        let from = Square((to_square.0 as i8 - forward_offset) as u8);
        v.push(Move::new_quiet(from, to_square));
        true
    });
    // Double pushes
    let double_push =
        pawn_single_push(single_push, props.not_occupied, props.color_to_move).and(props.to_mask);
    tos = if props.color_to_move == White {
        double_push.and(BitBoard::RANK_4)
    } else {
        double_push.and(BitBoard::RANK_5)
    };
    tos.for_each_set_bit(|to_square| {
        let from = Square((to_square.0 as i8 - 2 * forward_offset) as u8);
        v.push(Move::new_double_pawn_push(from, to_square));
        true
    });
    // Add pawn captures
    let add_pawn_captures = |tos: BitBoard, offset: i8, v: &mut Vec<Move>| {
        let tt = tos.and(props.attack_targets).and(props.to_mask);
        tt.and(promotion_rank).for_each_set_bit(|to_square| {
            let from = Square((to_square.0 as i8 + offset) as u8);
            PROMOTION_PIECES.iter().for_each(|p| {
                v.push(Move::new_promotion(from, to_square, true, *p));
            });
            true
        });
        tt.and(not_promotion_rank).for_each_set_bit(|to_square| {
            let from = Square((to_square.0 as i8 + offset) as u8);
            v.push(Move::new_capture(from, to_square));
            true
        });
        // Add en passant captures
        if props.en_passant.is_legal() && tos.is_set(props.en_passant.0) {
            let from = Square((props.en_passant.0 as i8 + offset) as u8);
            v.push(Move::new_en_passant(from, props.en_passant));
        }
    };
    add_pawn_captures(
        pawn_captures(props.pawns, props.color_to_move, true),
        -forward_offset - 1,
        v,
    );
    add_pawn_captures(
        pawn_captures(props.pawns, props.color_to_move, false),
        -forward_offset + 1,
        v,
    );
}

#[inline]
fn pawn_single_push(own_pawns: BitBoard, not_occupied: BitBoard, color_to_move: Color) -> BitBoard {
    let b = match color_to_move {
        White => own_pawns.shift_north(),
        _ => own_pawns.shift_south(),
    };
    b.and(not_occupied)
}

#[inline]
fn pawn_captures(own_pawns: BitBoard, color_to_move: Color, capture_east: bool) -> BitBoard {
    let b = match color_to_move {
        White => own_pawns.shift_north(),
        _ => own_pawns.shift_south(),
    };
    let b = if capture_east {
        b.shift_east()
    } else {
        b.shift_west()
    };
    b
}
#[inline]
fn pawn_captures_both(own_pawns: BitBoard, color_to_move: Color) -> BitBoard {
    pawn_captures(own_pawns, color_to_move, true).or(pawn_captures(own_pawns, color_to_move, false))
}

#[inline]
fn rook_attacks(rook_square: Square, occ: BitBoard) -> BitBoard {
    file_attacks(rook_square, occ).or(rank_attacks(rook_square, occ))
}

#[inline]
fn bishop_attacks(bishop_square: Square, occ: BitBoard) -> BitBoard {
    diagonal_attacks(bishop_square, occ).or(anti_diagonal_attacks(bishop_square, occ))
}

#[inline]
fn queen_attacks(queen_square: Square, occ: BitBoard) -> BitBoard {
    rook_attacks(queen_square, occ).or(bishop_attacks(queen_square, occ))
}

#[inline]
fn file_attacks(square: Square, occ: BitBoard) -> BitBoard {
    postive_ray_attacks(occ, Dir8::East, square).or(negative_ray_attacks(occ, Dir8::West, square))
}

#[inline]
fn rank_attacks(square: Square, occ: BitBoard) -> BitBoard {
    postive_ray_attacks(occ, Dir8::North, square).or(negative_ray_attacks(occ, Dir8::South, square))
}

#[inline]
fn diagonal_attacks(square: Square, occ: BitBoard) -> BitBoard {
    postive_ray_attacks(occ, Dir8::NorthEast, square).or(negative_ray_attacks(
        occ,
        Dir8::SouthWest,
        square,
    ))
}

#[inline]
fn anti_diagonal_attacks(square: Square, occ: BitBoard) -> BitBoard {
    postive_ray_attacks(occ, Dir8::NorthWest, square).or(negative_ray_attacks(
        occ,
        Dir8::SouthEast,
        square,
    ))
}

#[inline]
fn xray_rook(rook_square: Square, occ: BitBoard, blockers: BitBoard) -> BitBoard {
    let attacks = rook_attacks(rook_square, occ);
    attacks.xor(rook_attacks(rook_square, occ.xor(blockers.and(attacks))))
}

#[inline]
fn xray_bishop(bishop_square: Square, occ: BitBoard, blockers: BitBoard) -> BitBoard {
    let attacks = bishop_attacks(bishop_square, occ);
    attacks.xor(bishop_attacks(
        bishop_square,
        occ.xor(blockers.and(attacks)),
    ))
}

struct PinnedProps {
    king_square: Square,
    occupied: BitBoard,
    own_pieces: BitBoard,
    opp_rq: BitBoard,
    opp_bq: BitBoard,
}

/// Returns a `BitBoard` containing all squares with pinned pieces.
fn pinned(props: PinnedProps) -> BitBoard {
    let king_square = props.king_square;
    let occ = props.occupied;
    let own_pieces = props.own_pieces;
    let opp_rq = props.opp_rq;
    let opp_bq = props.opp_bq;
    let mut pinned = BitBoard::EMPTY;
    let pinners_rq = xray_rook(king_square, occ, own_pieces).and(opp_rq);
    let pinners_bq = xray_bishop(king_square, occ, own_pieces).and(opp_bq);
    pinners_rq.or(pinners_bq).for_each_set_bit(|square| {
        let p = between_bb(king_square, square).and(own_pieces);
        pinned = pinned.or(p);
        true
    });
    pinned
}

/// Returns a `BitBoard` containing all squares where the king would be attacked.
pub fn king_attack_map(board: &Board, opposing_color: Color) -> BitBoard {
    let mut map = BitBoard::EMPTY;
    // remove own king
    let occupied = board
        .occupied()
        .and(board.kings().and(board.own_color_board()).not());
    board
        .pieces(Piece::King, opposing_color)
        .for_each_set_bit(|king_square| {
            map = map.or(king_moves(king_square));
            true
        });
    board
        .pieces(Piece::Queen, opposing_color)
        .for_each_set_bit(|queen_square| {
            map = map.or(queen_attacks(queen_square, occupied));
            true
        });
    board
        .pieces(Piece::Rook, opposing_color)
        .for_each_set_bit(|rook_square| {
            map = map.or(rook_attacks(rook_square, occupied));
            true
        });
    board
        .pieces(Piece::Bishop, opposing_color)
        .for_each_set_bit(|bishop_square| {
            map = map.or(bishop_attacks(bishop_square, occupied));
            true
        });
    board
        .pieces(Piece::Knight, opposing_color)
        .for_each_set_bit(|knight_square| {
            map = map.or(knight_attacks(knight_square));
            true
        });
    let pawns = board.pieces(Piece::Pawn, opposing_color);
    map = map.or(pawn_captures_both(pawns, opposing_color));
    map
}

struct AttacksToKingProps {
    king_square: Square,
    color_to_move: Color,
    occupied: BitBoard,
    opp_color_board: BitBoard,
    pieces: [BitBoard; Piece::COUNT],
}

/// Returns a `BitBoard` containing all pieces currently attacking the king.
///
/// This works by putting every piece type at the king's square and checking if that
/// attacks a piece of its own type.
fn attacks_to_king(props: AttacksToKingProps) -> BitBoard {
    let opp_board = props.opp_color_board;
    let king_square = props.king_square;
    let occ = props.occupied;
    let pieces = props.pieces;
    let mut attacks = BitBoard::EMPTY;
    attacks =
        attacks.or(queen_attacks(king_square, occ).and(pieces[Piece::Queen.idx()].and(opp_board)));
    attacks =
        attacks.or(rook_attacks(king_square, occ).and(pieces[Piece::Rook.idx()].and(opp_board)));
    attacks = attacks
        .or(bishop_attacks(king_square, occ).and(pieces[Piece::Bishop.idx()].and(opp_board)));
    attacks =
        attacks.or(knight_attacks(king_square).and(pieces[Piece::Knight.idx()].and(opp_board)));
    attacks = attacks.or(pawn_captures_both(
        BitBoard::from_square(king_square),
        props.color_to_move,
    )
    .and(pieces[Piece::Pawn.idx()].and(opp_board)));
    attacks
}

fn postive_ray_attacks(occ: BitBoard, ray: Dir8, square: Square) -> BitBoard {
    let attacks = ray_attacks(square, ray);
    let blocker = occ.and(attacks);
    if blocker.is_not_empty() {
        let b = blocker.bit_scan_forward();
        return attacks.xor(ray_attacks(Square(b), ray));
    }
    attacks
}

fn negative_ray_attacks(occ: BitBoard, ray: Dir8, square: Square) -> BitBoard {
    let attacks = ray_attacks(square, ray);
    let blocker = occ.and(attacks);
    if blocker.is_not_empty() {
        let b = blocker.bit_scan_backward();
        return attacks.xor(ray_attacks(Square(b), ray));
    }
    attacks
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::{Color, Piece};
    use crate::search::find_best_move;
    use crate::util::assert_eq_unordered;
    use rand::prelude::IndexedRandom;
    use std::collections::BTreeSet;

    fn move_list(list: &[&str]) -> Vec<Move> {
        list.iter()
            .copied()
            .map(|m| m.parse::<Move>().unwrap())
            .collect()
    }
    fn assert_move_sources(moves: &[Move], expected: &[&str]) {
        let actual: BTreeSet<String> = moves.iter().map(|m| m.from().to_string()).collect();
        let expected: BTreeSet<String> = expected.iter().map(|&s| s.to_string()).collect();
        assert_eq!(actual, expected);
    }
    fn assert_move_destinations(moves: &[Move], expected: &[&str]) {
        let actual: BTreeSet<String> = moves.iter().map(|m| m.to().to_string()).collect();
        let expected: BTreeSet<String> = expected.iter().map(|&s| s.to_string()).collect();
        assert_eq!(actual, expected);
    }
    fn assert_eq_moves(a: &[Move], b: &[Move]) {
        let a: Vec<String> = a.iter().map(|m| m.to_string()).collect();
        let b: Vec<String> = b.iter().map(|m| m.to_string()).collect();
        assert_eq_unordered(&a, &b);
    }
    #[test]
    fn xray_rook_should_be_correct() {
        let blockers = BitBoard::try_from_coords(["a3", "a6"]).unwrap();
        let xray = xray_rook(Square(0), blockers, blockers);
        assert_eq!(xray, BitBoard::try_from_coords(["a4", "a5", "a6"]).unwrap());
    }
    #[test]
    fn king_should_not_capture_own_piece() {
        let board = Board::from_fen("8/8/8/8/8/p1k5/P7/K7 w - - 0 1");
        assert_eq_moves(&generate_moves(&board), &move_list(&["a1b1"]));
    }
    #[test]
    fn rook_should_move_correctly_from_a1() {
        let board = Board::from_fen("5k1K/6r1/8/8/8/8/8/R7 w - - 0 1");
        let moves = generate_moves(&board);
        assert_move_sources(&moves, &["a1"]);
        assert_move_destinations(
            &moves,
            &[
                "a2", "a3", "a4", "a5", "a6", "a7", "a8", "b1", "c1", "d1", "e1", "f1", "g1", "h1",
            ],
        );
    }
    #[test]
    fn black_rook_should_move_correctly_from_e4() {
        let board = Board::from_fen("5K1k/6R1/8/8/4r3/8/8/8 b - - 0 1");
        let moves = generate_moves(&board);
        assert_move_sources(&moves, &["e4"]);
        assert_move_destinations(
            &moves,
            &[
                "e1", "e2", "e3", "e5", "e6", "e7", "e8", "a4", "b4", "c4", "d4", "f4", "g4", "h4",
            ],
        );
    }
    #[test]
    fn queen_should_move_correctly() {
        let board = Board::from_fen("8/8/3p4/3P4/p7/8/Q3p3/K1k5 w - - 0 1");
        assert_eq_moves(
            &generate_moves(&board),
            &move_list(&[
                "a2a3", "a2a4", "a2b3", "a2c4", "a2b2", "a2c2", "a2d2", "a2e2", "a2b1",
            ]),
        );
    }
    #[test]
    fn rook_should_move_correctly() {
        let board = Board::from_fen("8/8/p7/P7/8/R2r4/1r6/K1k5 w - - 0 1");
        assert_eq_moves(
            &generate_moves(&board),
            &move_list(&["a3a2", "a3a4", "a3b3", "a3c3", "a3d3"]),
        );
    }
    #[test]
    fn bishop_should_move_correctly() {
        let board = Board::from_fen("8/3p4/3P4/8/1B6/p7/P1kn4/K7 w - - 0 1");
        assert_eq_moves(
            &generate_moves(&board),
            &move_list(&["b4a3", "b4a5", "b4c5", "b4c3", "b4d2"]),
        );
    }
    #[test]
    fn knight_should_move_correctly() {
        let board = Board::from_fen("1r5k/8/8/8/8/p7/P3r3/K1N5 w - - 0 1");
        assert_eq_moves(
            &generate_moves(&board),
            &move_list(&["c1b3", "c1d3", "c1e2"]),
        );
    }
    #[test]
    fn knight_should_resolve_check() {
        let board = Board::from_fen("r6k/2N5/1r6/8/8/8/8/K7 w - - 0 1");
        assert_eq_moves(&generate_moves(&board), &move_list(&["c7a8", "c7a6"]));
    }
    #[test]
    fn pawns_should_find_moves() {
        let board = Board::from_fen("7k/5P2/8/8/8/r1r5/1P6/1K6 w - - 0 1");
        assert_eq_moves(
            &generate_moves(&board),
            &move_list(&[
                "b2b3", "b2b4", "b2a3", "b2c3", "f7f8=Q", "f7f8=N", "f7f8=R", "f7f8=B",
            ]),
        );
    }
    #[test]
    fn should_find_no_moves_when_checkmate() {
        let mut board = Board::empty()
            .set_piece("e6".parse().unwrap(), Piece::King, Color::White)
            .set_piece("a8".parse().unwrap(), Piece::Rook, Color::White)
            .set_piece("e8".parse().unwrap(), Piece::King, Color::Black)
            .set_color_to_move(Color::Black);
        let moves = generate_moves(&board);
        assert_eq!(moves.len(), 0);
    }
    #[test]
    fn should_respect_pin_by_queens() {
        let board = Board::from_fen("1q2q2q/6R1/3RB3/q2BKB1q/3RBR2/8/1q5q/k3q3 w - - 0 1");
        assert_eq_moves(&generate_moves(&board), &move_list(&["e5f6"]));
    }
    #[test]
    fn should_respect_pin_by_rook() {
        let board = Board::from_fen("8/8/8/8/8/8/5kr1/5rRK w - - 0 1");
        assert_eq_moves(&generate_moves(&board), &move_list(&["g1f1"]));
    }
    #[test]
    fn should_respect_pins_by_bishops() {
        let board = Board::from_fen("1q2q2q/6R1/3RB3/q2BKB1q/3RBR2/8/1q5q/k3q3 w - - 0 1");
        assert_eq_moves(&generate_moves(&board), &move_list(&["e5f6"]));
    }

    #[test]
    fn when_in_check_should_evade() {
        let board = Board::from_fen("7k/8/8/8/1R6/8/8/Kr6 w - - 0 1");
        assert_eq_moves(
            &generate_moves(&board),
            &move_list(&["a1a2", "a1b1", "b4b1"]),
        );
    }
    #[test]
    fn when_in_check_should_block() {
        let board = Board::from_fen("7k/8/8/8/8/8/1R6/K1r5 w - - 0 1");
        assert_eq_unordered(&generate_moves(&board), &move_list(&["a1a2", "b2b1"]));
    }
    #[test]
    fn should_solve_king_check_position() {
        let board = Board::from_fen("7k/8/8/8/1RR5/8/8/K1r3R1 w - - 0 1");
        assert_eq_moves(
            &generate_moves(&board),
            &move_list(&["a1a2", "a1b2", "b4b1", "c4c1", "g1c1"]),
        );
    }

    #[test]
    fn should_solve_double_check_position() {
        let board = Board::from_fen("7k/3r2R1/8/8/5R2/8/8/3K2r1 w - - 0 1");
        assert_eq_moves(&generate_moves(&board), &move_list(&["d1c2", "d1e2"]));
    }

    #[test]
    fn white_castling_moves_should_be_found() {
        let board = Board::from_fen("8/8/8/8/8/4k3/P6P/R3K2R w KQ - 0 1");
        assert_eq_moves(
            &generate_moves(&board),
            &move_list(&[
                "a1b1", "a1c1", "a1d1", "a2a3", "a2a4", "e1c1", "e1d1", "e1f1", "e1g1", "h1f1",
                "h1g1", "h2h3", "h2h4",
            ]),
        );
    }

    #[test]
    fn should_support_en_passant() {
        let mut board = Board::from_fen("7k/8/7K/8/1p6/8/P7/8 w - - 0 1");
        println!("board:\n{}", board);
        board.make_move(Move::new_double_pawn_push(Square::A2, Square::A4));
        assert_eq!(board.en_passant, Square::A3);
        assert_eq_moves(
            &generate_moves(&board),
            &move_list(&["b4b3", "b4a3", "h8g8"]),
        );
    }

    #[test]
    fn should_find_correct_moves_1() {
        let board =
            Board::from_fen("rnbqkbnr/2p1pppp/1p6/pB1p4/4P3/2N2N2/PPPP1PPP/R1BQK2R b - - 0 1");
        assert_eq_moves(
            &generate_moves(&board),
            &move_list(&["b8c6", "b8d7", "c7c6", "c8d7", "d8d7"]),
        );
    }
    #[test]
    fn case_2() {
        let mut board = Board::from_fen("r1b1r3/pp1pkppP/2npp3/8/3P4/2P1NN2/PP4R1/R6K w - - 1 3");
        find_best_move(&mut board, 2);
        // should not panic!
    }

    #[test]
    fn random_game_two_kings() {
        use rand::SeedableRng;
        let mut board = Board::from_fen("4k3/8/8/8/8/8/8/4K3 w - - 0 1");
        println!("\nInitial position:");
        println!("{}", board);
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        // Total of 10 ply moves
        for move_num in 1..=10 {
            let moves = generate_moves(&board);
            println!("Pseudo-Legal Moves: {:?}", moves);
            if let Some(white_move) = moves.choose(&mut rng) {
                println!(
                    "Ply {}: {} plays {} -> {}",
                    move_num,
                    board.color_to_move(),
                    white_move.from().to_string(),
                    white_move.to().to_string()
                );
                board.make_move(*white_move).unwrap();
                println!("{}", board);
            }
        }
    }

    #[test]
    fn best_game() {
        let mut board = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        println!("\nInitial position:");
        println!("{}", board);
        // Total of 10 ply moves
        for move_num in 1..=120 {
            let best_move = find_best_move(&mut board, 2)
                .move_
                .unwrap_or_else(|| panic!("Found no move"));
            println!(
                "Ply {}: {} plays {} -> {}",
                move_num,
                board.color_to_move(),
                best_move.from().to_string(),
                best_move.to().to_string()
            );
            board.make_move(best_move).unwrap();
            println!("{}", board);
        }
    }
}
