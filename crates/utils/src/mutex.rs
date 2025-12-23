//! Utilities for deadlock-safe mutex locking.
//!
//! This module provides utilities for acquiring multiple mutex locks in a consistent
//! order to prevent deadlocks. When two threads try to lock the same pair of mutexes
//! in different orders, a deadlock can occur. The [`lock_in_ptr_order`] function
//! ensures consistent ordering based on memory addresses.

use std::sync::{Arc, Mutex, MutexGuard};

/// Locks two mutexes in a consistent order based on their memory addresses.
///
/// This function prevents deadlocks that can occur when two threads try to lock
/// the same pair of mutexes in different orders. By always locking in pointer order,
/// a consistent global ordering is maintained.
///
/// # Arguments
///
/// * `first` - The first mutex to lock
/// * `second` - The second mutex to lock
///
/// # Returns
///
/// A tuple of `MutexGuard`s in the same order as the arguments (first, second),
/// regardless of which mutex was locked first internally.
///
/// # Panics
///
/// Panics if `first` and `second` point to the same mutex, as locking the same
/// mutex twice would cause a deadlock.
///
/// # Examples
///
/// ```
/// use std::sync::{Arc, Mutex};
/// use cryfs_utils::mutex::lock_in_ptr_order;
///
/// let mutex_a = Arc::new(Mutex::new(1));
/// let mutex_b = Arc::new(Mutex::new(2));
///
/// let (guard_a, guard_b) = lock_in_ptr_order(&mutex_a, &mutex_b);
/// assert_eq!(*guard_a, 1);
/// assert_eq!(*guard_b, 2);
/// ```
pub fn lock_in_ptr_order<'a, 'b, T>(
    first: &'a Arc<Mutex<T>>,
    second: &'b Arc<Mutex<T>>,
) -> (MutexGuard<'a, T>, MutexGuard<'b, T>) {
    let first_ptr = Arc::as_ptr(first);
    let second_ptr = Arc::as_ptr(second);
    assert_ne!(first_ptr, second_ptr, "Mutexes must not be the same");

    if first_ptr < second_ptr {
        let first = first.lock().unwrap();
        let second = second.lock().unwrap();
        (first, second)
    } else {
        let second = second.lock().unwrap();
        let first = first.lock().unwrap();
        (first, second)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_locks_both_mutexes() {
        let mutex_a = Arc::new(Mutex::new(1));
        let mutex_b = Arc::new(Mutex::new(2));

        let (guard_a, guard_b) = lock_in_ptr_order(&mutex_a, &mutex_b);

        assert_eq!(*guard_a, 1);
        assert_eq!(*guard_b, 2);
    }

    #[test]
    fn test_consistent_order_regardless_of_argument_order() {
        let mutex_a = Arc::new(Mutex::new(1));
        let mutex_b = Arc::new(Mutex::new(2));

        // Lock in one order
        {
            let (guard_a, guard_b) = lock_in_ptr_order(&mutex_a, &mutex_b);
            assert_eq!(*guard_a, 1);
            assert_eq!(*guard_b, 2);
        }

        // Lock in reverse order - should still work without deadlock
        {
            let (guard_b, guard_a) = lock_in_ptr_order(&mutex_b, &mutex_a);
            assert_eq!(*guard_a, 1);
            assert_eq!(*guard_b, 2);
        }
    }

    #[test]
    fn test_can_modify_through_guards() {
        let mutex_a = Arc::new(Mutex::new(1));
        let mutex_b = Arc::new(Mutex::new(2));

        {
            let (mut guard_a, mut guard_b) = lock_in_ptr_order(&mutex_a, &mutex_b);
            *guard_a = 10;
            *guard_b = 20;
        }

        assert_eq!(*mutex_a.lock().unwrap(), 10);
        assert_eq!(*mutex_b.lock().unwrap(), 20);
    }

    #[test]
    #[should_panic(expected = "Mutexes must not be the same")]
    fn test_panics_on_same_mutex() {
        let mutex = Arc::new(Mutex::new(1));
        let _ = lock_in_ptr_order(&mutex, &mutex);
    }
}
