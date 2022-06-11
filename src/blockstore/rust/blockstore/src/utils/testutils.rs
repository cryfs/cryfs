use std::fmt::Debug;

pub fn assert_unordered_vec_eq<T: Eq + Ord + Debug>(mut lhs: Vec<T>, mut rhs: Vec<T>) {
    lhs.sort();
    rhs.sort();
    assert_eq!(lhs, rhs);
}
