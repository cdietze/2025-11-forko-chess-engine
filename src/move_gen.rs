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
pub fn to_index(coords: &str) -> Option<SquareIndex> {
    let b = coords.as_bytes();
    if b.len() != 2 { return None; }
    let file = match b[0] { b'a'..=b'h' => b[0] - b'a', b'A'..=b'H' => b[0] - b'A', _ => return None };
    let rank = match b[1] { b'1'..=b'8' => b[1] - b'1', _ => return None };
    Some(rank * 8 + file)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn print_king_attacks_from_g2() {
        let square = "g2";
        let king = single_square(to_index(square).unwrap());
        let attacks = king_attacks(king);
        println!("Input (king on {}):\n{}", square, print_bb(king));
        println!("King attacks from {}:\n{}", square, print_bb(attacks));
    }

    #[test]
    fn test_to_index_corners() {
        assert_eq!(to_index("a1"), Some(0));
        assert_eq!(to_index("h1"), Some(7));
        assert_eq!(to_index("a8"), Some(56));
        assert_eq!(to_index("h8"), Some(63));
    }

    #[test]
    fn test_to_index_examples() {
        assert_eq!(to_index("b1"), Some(1));
        assert_eq!(to_index("g2"), Some(14));
        assert_eq!(to_index("d4"), Some(27));
        assert_eq!(to_index("e5"), Some(36));
    }
    #[test]
    fn test_to_index_negative_examples() {
        assert_eq!(to_index(""), None);
        assert_eq!(to_index("a1a"), None);
        assert_eq!(to_index("1a"), None);
        assert_eq!(to_index("x1"), None);
        assert_eq!(to_index("a0"), None);
        assert_eq!(to_index("a9"), None);
    }
}
