use std::sync::{Arc, Mutex, Weak};

/// A lazy, shared, on-demand initialized value that is dropped when no longer in use.
///
/// Similar to [`std::sync::LazyLock`], this creates a shared instance where every
/// caller accesses the same value. However, unlike `LazyLock`:
///
/// - The value is dropped when the last [`Arc`] reference is dropped
/// - If requested again after being dropped, a **new** instance is created
///
/// This is useful for resources that are expensive to create but should not be
/// kept alive indefinitely when not in use (e.g., thread pools, caches).
///
/// # Thread Safety
///
/// `LazyReclaim` is thread-safe. Multiple threads can call [`get_or_init()`](Self::get_or_init)
/// concurrently, and they will all receive the same [`Arc`] pointing to the same instance.
///
/// # Example
///
/// ```
/// use std::sync::Arc;
/// use cryfs_utils::lazy_reclaim::LazyReclaim;
///
/// static POOL: LazyReclaim<Vec<u8>> = LazyReclaim::new(|| {
///     vec![1, 2, 3] // Expensive initialization
/// });
///
/// // First call initializes the value
/// let arc1 = POOL.get_or_init();
/// let arc2 = POOL.get_or_init();
/// assert!(Arc::ptr_eq(&arc1, &arc2)); // Same instance
///
/// // After all Arcs are dropped, the value is dropped
/// drop(arc1);
/// drop(arc2);
///
/// // Next call creates a new instance
/// let arc3 = POOL.get_or_init();
/// ```
pub struct LazyReclaim<T> {
    inner: Mutex<Weak<T>>,
    init: fn() -> T,
}

impl<T> LazyReclaim<T> {
    /// Creates a new `LazyReclaim` with the given initialization function.
    ///
    /// This is a `const fn`, so it can be used to initialize `static` variables.
    ///
    /// The `init` function will be called lazily on the first call to [`get_or_init()`](Self::get_or_init),
    /// and again each time the value needs to be re-initialized after being dropped.
    pub const fn new(init: fn() -> T) -> Self {
        Self {
            inner: Mutex::new(Weak::new()),
            init,
        }
    }

    /// Returns a shared reference to the value, initializing it if necessary.
    ///
    /// If the value exists (i.e., there are other [`Arc`] references keeping it alive),
    /// returns a new [`Arc`] pointing to the same instance.
    ///
    /// If the value has been dropped (all previous [`Arc`] references were dropped),
    /// calls the `init` function to create a new instance.
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned (a previous holder panicked).
    pub fn get_or_init(&self) -> Arc<T> {
        let mut weak = self.inner.lock().unwrap();
        weak.upgrade().unwrap_or_else(|| {
            let arc = Arc::new((self.init)());
            *weak = Arc::downgrade(&arc);
            arc
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[test]
    fn test_get_returns_value() {
        let lazy = LazyReclaim::new(|| 42);
        let arc = lazy.get_or_init();
        assert_eq!(*arc, 42);
    }

    #[test]
    fn test_multiple_gets_return_same_instance() {
        let lazy = LazyReclaim::new(|| 42);
        let arc1 = lazy.get_or_init();
        let arc2 = lazy.get_or_init();
        assert!(Arc::ptr_eq(&arc1, &arc2));
    }

    #[test]
    fn test_init_called_once_while_held() {
        static INIT_COUNT: AtomicU32 = AtomicU32::new(0);

        fn init() -> u32 {
            INIT_COUNT.fetch_add(1, Ordering::SeqCst)
        }

        let lazy = LazyReclaim::new(init);

        let arc1 = lazy.get_or_init();
        let arc2 = lazy.get_or_init();
        let arc3 = lazy.get_or_init();

        assert_eq!(INIT_COUNT.load(Ordering::SeqCst), 1);
        assert!(Arc::ptr_eq(&arc1, &arc2));
        assert!(Arc::ptr_eq(&arc2, &arc3));
    }

    #[test]
    fn test_reinitializes_after_all_refs_dropped() {
        static INIT_COUNT: AtomicU32 = AtomicU32::new(0);

        fn init() -> u32 {
            INIT_COUNT.fetch_add(1, Ordering::SeqCst)
        }

        let lazy = LazyReclaim::new(init);

        // First get
        let arc1 = lazy.get_or_init();
        assert_eq!(*arc1, 0); // First init returns 0
        assert_eq!(INIT_COUNT.load(Ordering::SeqCst), 1);

        // Drop and get again
        drop(arc1);
        let arc2 = lazy.get_or_init();
        assert_eq!(*arc2, 1); // Second init returns 1
        assert_eq!(INIT_COUNT.load(Ordering::SeqCst), 2);

        // Drop and get again
        drop(arc2);
        let arc3 = lazy.get_or_init();
        assert_eq!(*arc3, 2); // Third init returns 2
        assert_eq!(INIT_COUNT.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn test_keeps_alive_while_any_ref_exists() {
        static INIT_COUNT: AtomicU32 = AtomicU32::new(0);

        fn init() -> u32 {
            INIT_COUNT.fetch_add(1, Ordering::SeqCst)
        }

        let lazy = LazyReclaim::new(init);

        let arc1 = lazy.get_or_init();
        let arc2 = lazy.get_or_init();

        // Drop one, but other still holds
        drop(arc1);
        let arc3 = lazy.get_or_init();

        // Should still be the same instance
        assert!(Arc::ptr_eq(&arc2, &arc3));
        assert_eq!(INIT_COUNT.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_works_in_static() {
        static LAZY: LazyReclaim<u32> = LazyReclaim::new(|| 123);

        let arc1 = LAZY.get_or_init();
        let arc2 = LAZY.get_or_init();

        assert_eq!(*arc1, 123);
        assert!(Arc::ptr_eq(&arc1, &arc2));
    }

    #[test]
    fn test_thread_safety() {
        use std::thread;

        static LAZY: LazyReclaim<u32> = LazyReclaim::new(|| 999);

        let handles: Vec<_> = (0..10)
            .map(|_| {
                thread::spawn(|| {
                    let arc = LAZY.get_or_init();
                    assert_eq!(*arc, 999);
                    arc
                })
            })
            .collect();

        let arcs: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();

        // All threads should have gotten the same instance
        for arc in &arcs {
            assert!(Arc::ptr_eq(arc, &arcs[0]));
        }
    }
}
