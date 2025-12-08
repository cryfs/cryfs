use std::fmt::Debug;
use std::hash::Hash;

// TODO Once stable, we could consider using rust's [std::iter::Step] trait instead.
pub trait HandleTrait: Clone + Eq + Ord + Hash + Debug {
    const MIN: Self;
    const MAX: Self;

    fn incremented(&self) -> Self;
    fn range(begin_inclusive: &Self, end_exclusve: &Self) -> impl Iterator<Item = Self>;
}
