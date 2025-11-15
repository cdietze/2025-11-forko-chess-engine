pub fn assert_eq_unordered<T: Ord + std::fmt::Debug + Clone>(a: &[T], b: &[T]) {
    let mut a = a.to_vec();
    let mut b = b.to_vec();
    a.sort();
    b.sort();
    assert_eq!(a, b);
}
