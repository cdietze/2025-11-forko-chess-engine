use crate::board::{BB, SquareIndex};
fn king_attacks(b: BB) -> BB {
    let mut r = b | shift_east(b) | shift_west(b);
    r |= shift_north(r) | shift_south(r);
    r &= !b;
    r
}

fn shift_south(b: BB) -> BB {
    b >> 8
}
fn shift_north(b: BB) -> BB {
    b << 8
}

fn shift_east(b: BB) -> BB {
    (b << 1) & NOT_A_FILE
}
fn shift_west(b: BB) -> BB {
    (b >> 1) & NOT_H_FILE
}

const NOT_A_FILE: BB = 0xfefefefefefefefe;
const NOT_H_FILE: BB = 0x7f7f7f7f7f7f7f7f;
const EMPTY: BB = 0;
const FULL: BB = 0xffffffffffffffff;

fn single_square(i: SquareIndex) -> BB {
    1u64 << i
}

fn print_bb(b: BB) -> String {
    let mut result = String::new();
    for rank in (0..8).rev() {
        for file in 0..8 {
            let square = rank * 8 + file;
            result.push(' ');
            result.push(if (b & (1u64 << square)) == 0 {
                '.'
            } else {
                '1'
            });
        }
        result.push('\n');
    }
    result
}

/**
Converts given `coords` into its index on the board.
E.g.
"a1" -> 0
"b1" -> 1
"h8" -> 63
*/
fn to_index(coords: &str) -> SquareIndex {
    let bytes = coords.as_bytes();
    let file = (bytes[0] - b'a') as SquareIndex;
    let rank = (bytes[1] - b'1') as SquareIndex;
    rank * 8 + file
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn print_king_attacks_from_g2() {
        let square = "g2";
        let king = single_square(to_index(square));
        let attacks = king_attacks(king);
        println!("Input (king on {}):\n{}", square, print_bb(king));
        println!("King attacks from {}:\n{}", square, print_bb(attacks));
    }

    #[test]
    fn test_to_index_corners() {
        assert_eq!(to_index("a1"), 0);
        assert_eq!(to_index("h1"), 7);
        assert_eq!(to_index("a8"), 56);
        assert_eq!(to_index("h8"), 63);
    }

    #[test]
    fn test_to_index_examples() {
        assert_eq!(to_index("b1"), 1);
        assert_eq!(to_index("g2"), 14);
        assert_eq!(to_index("d4"), 27);
        assert_eq!(to_index("e5"), 36);
    }
}
