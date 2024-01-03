use std::fmt::Debug;
use std::ops::{Bound, RangeBounds};

#[track_caller]
pub fn assert_unordered_vec_eq<T: Eq + Ord + Debug>(mut lhs: Vec<T>, mut rhs: Vec<T>) {
    lhs.sort();
    rhs.sort();
    if lhs != rhs {
        let (left_only, right_only, both) = difference_partition(lhs, rhs);
        panic!(
            "Assertion failed: Vec's are different.\n- both ({both_len}): {both:?}\n- left only ({left_only_len}): {left_only:?}\n- right only ({right_only_len}): {right_only:?}",
            both_len = both.len(),
            left_only_len = left_only.len(),
            right_only_len = right_only.len(),
        );
    }
}

/// Take `lhs` and `rhs` and return three vectors: (`lhs_only`, `rhs_only`, `both`)
/// with the elements that are only in `lhs`, only in `rhs` or in both.
fn difference_partition<T: PartialEq + Eq>(
    mut lhs: Vec<T>,
    mut rhs: Vec<T>,
) -> (Vec<T>, Vec<T>, Vec<T>) {
    let mut both = vec![];
    lhs.retain(|x| {
        if let Some(rhs_pos) = rhs.iter().position(|y| *y == *x) {
            let x = rhs.remove(rhs_pos);
            both.push(x);
            false
        } else {
            // Keep the entry in `lhs`
            true
        }
    });
    (lhs, rhs, both)
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
#[cfg(test)]
mod tests {
    use super::*;

    mod difference_partition {
        use super::*;

        #[test]
        fn empty() {
            let (lhs_only, rhs_only, both) = difference_partition::<u8>(vec![], vec![]);
            assert_eq!(lhs_only, vec![]);
            assert_eq!(rhs_only, vec![]);
            assert_eq!(both, vec![]);
        }

        #[test]
        fn lhs_only() {
            let (lhs_only, rhs_only, both) = difference_partition(vec![1, 2, 3], vec![]);
            assert_eq!(lhs_only, vec![1, 2, 3]);
            assert_eq!(rhs_only, vec![]);
            assert_eq!(both, vec![]);
        }

        #[test]
        fn rhs_only() {
            let (lhs_only, rhs_only, both) = difference_partition(vec![], vec![1, 2, 3]);
            assert_eq!(lhs_only, vec![]);
            assert_eq!(rhs_only, vec![1, 2, 3]);
            assert_eq!(both, vec![]);
        }

        #[test]
        fn both() {
            let (lhs_only, rhs_only, both) = difference_partition(vec![1, 2, 3], vec![2, 3, 1]);
            assert_eq!(lhs_only, vec![]);
            assert_eq!(rhs_only, vec![]);
            assert_eq!(both, vec![1, 2, 3]);
        }

        #[test]
        fn lhs_and_both() {
            let (lhs_only, rhs_only, both) = difference_partition(vec![1, 2, 3], vec![2, 3]);
            assert_eq!(lhs_only, vec![1]);
            assert_eq!(rhs_only, vec![]);
            assert_eq!(both, vec![2, 3]);
        }

        #[test]
        fn rhs_and_both() {
            let (lhs_only, rhs_only, both) = difference_partition(vec![1, 2], vec![2, 3, 1]);
            assert_eq!(lhs_only, vec![]);
            assert_eq!(rhs_only, vec![3]);
            assert_eq!(both, vec![1, 2]);
        }

        #[test]
        fn lhs_and_rhs() {
            let (lhs_only, rhs_only, both) = difference_partition(vec![1, 2], vec![3, 4]);
            assert_eq!(lhs_only, vec![1, 2]);
            assert_eq!(rhs_only, vec![3, 4]);
            assert_eq!(both, vec![]);
        }

        #[test]
        fn lhs_and_rhs_and_both() {
            let (lhs_only, rhs_only, both) = difference_partition(vec![1, 2, 3], vec![3, 4, 1]);
            assert_eq!(lhs_only, vec![2]);
            assert_eq!(rhs_only, vec![4]);
            assert_eq!(both, vec![1, 3]);
        }

        #[test]
        fn lhs_and_rhs_and_both_and_more() {
            let (lhs_only, rhs_only, both) =
                difference_partition(vec![1, 2, 3, 5], vec![3, 4, 1, 6]);
            assert_eq!(lhs_only, vec![2, 5]);
            assert_eq!(rhs_only, vec![4, 6]);
            assert_eq!(both, vec![1, 3]);
        }

        #[test]
        fn duplicate_entries() {
            let (lhs_only, rhs_only, both) =
                difference_partition(vec![1, 2, 3, 3], vec![3, 1, 4, 1, 6]);
            assert_eq!(lhs_only, vec![2, 3]);
            assert_eq!(rhs_only, vec![4, 1, 6]);
            assert_eq!(both, vec![1, 3]);
        }

        #[test]
        fn more_duplicate_entries() {
            let (lhs_only, rhs_only, both) =
                difference_partition(vec![1, 2, 2, 3, 3, 3, 3], vec![3, 3, 1, 4, 4, 1, 6]);
            assert_eq!(lhs_only, vec![2, 2, 3, 3]);
            assert_eq!(rhs_only, vec![4, 4, 1, 6]);
            assert_eq!(both, vec![1, 3, 3]);
        }
    }
}
