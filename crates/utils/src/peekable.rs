//! Extension traits for [`Peekable`] iterators.
//!
//! This module provides the [`PeekableExt`] trait which adds convenience methods
//! to [`Peekable`] iterators.

use std::iter::Peekable;

/// Extension trait for [`Peekable`] iterators that provides additional utility methods.
pub trait PeekableExt {
    /// Returns `true` if the iterator has no more elements.
    ///
    /// This is equivalent to checking if `peek()` returns `None`, but provides
    /// a more readable API.
    ///
    /// # Examples
    ///
    /// ```
    /// use cryfs_utils::peekable::PeekableExt;
    ///
    /// let mut iter = [1, 2, 3].into_iter().peekable();
    /// assert!(!iter.is_empty());
    ///
    /// // Consume all elements
    /// iter.next();
    /// iter.next();
    /// iter.next();
    ///
    /// assert!(iter.is_empty());
    /// ```
    fn is_empty(&mut self) -> bool;
}

impl<T: Iterator> PeekableExt for Peekable<T> {
    #[inline]
    fn is_empty(&mut self) -> bool {
        self.peek().is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_empty_on_empty_iterator() {
        let mut iter = std::iter::empty::<i32>().peekable();
        assert!(iter.is_empty());
    }

    #[test]
    fn test_is_empty_on_nonempty_iterator() {
        let mut iter = [1, 2, 3].into_iter().peekable();
        assert!(!iter.is_empty());
    }

    #[test]
    fn test_is_empty_after_consuming_all_items() {
        let mut iter = [1].into_iter().peekable();
        assert!(!iter.is_empty());

        iter.next();
        assert!(iter.is_empty());
    }

    #[test]
    fn test_is_empty_does_not_consume_elements() {
        let mut iter = [1, 2].into_iter().peekable();

        // Check twice - should still have elements
        assert!(!iter.is_empty());
        assert!(!iter.is_empty());

        // First element should still be available
        assert_eq!(Some(1), iter.next());
    }
}
