use std::sync::{Arc, Mutex, MutexGuard};

/// This function can be used to lock two [Mutex]es in a consistent order.
/// If two mutexes are locked in arbitrary orders by different threads, this can lead to deadlocks.
/// This function can be used to avoid this problem.
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

// TODO Tests
