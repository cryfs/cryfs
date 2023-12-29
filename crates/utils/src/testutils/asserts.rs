use std::fmt::Debug;
use std::ops::{Bound, RangeBounds};

#[track_caller]
pub fn assert_unordered_vec_eq<T: Eq + Ord + Debug>(mut lhs: Vec<T>, mut rhs: Vec<T>) {
    lhs.sort();
    rhs.sort();
    if lhs != rhs {
        let left_only = lhs.iter().filter(|x| !rhs.contains(x)).collect::<Vec<_>>();
        let right_only = rhs.iter().filter(|x| !lhs.contains(x)).collect::<Vec<_>>();
        let both = lhs.iter().filter(|x| rhs.contains(x)).collect::<Vec<_>>();
        panic!(
            "Assertion failed: Vec's are different.\n- both ({both_len}): {both:?}\n- left only ({left_only_len}): {left_only:?}\n- right only ({right_only_len}): {right_only:?}",
            both_len = both.len(),
            left_only_len = left_only.len(),
            right_only_len = right_only.len(),
        );
    }
}

#[track_caller]
pub fn assert_data_range_eq(lhs: &[u8], rhs: &[u8], range: impl RangeBounds<usize>) {
    assert_eq!(_apply_bound(lhs, &range), _apply_bound(rhs, &range));
}

fn _apply_bound<'a, 'b>(data: &'a [u8], range: &'b impl RangeBounds<usize>) -> &'a [u8] {
    let start = match range.start_bound() {
        Bound::Unbounded => 0,
        Bound::Included(&x) => x,
        Bound::Excluded(&x) => x + 1,
    };
    let end = match range.end_bound() {
        Bound::Unbounded => data.len(),
        Bound::Included(&x) => x + 1,
        Bound::Excluded(&x) => x,
    };
    &data[start..end]
}
