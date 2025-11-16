pub fn assert_eq_unordered<T: Ord + std::fmt::Debug + Clone>(a: &[T], b: &[T]) {
    let mut a = a.to_vec();
    let mut b = b.to_vec();
    a.sort();
    b.sort();
    assert_eq!(a, b);
}

pub fn with_separator(n: i32) -> String {
    let s = n.to_string();
    s.as_bytes()
        .rchunks(3)
        .rev()
        .map(std::str::from_utf8)
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
        .join("_")
}
