//! Run `Drop` for values stored in a `static` at process exit.
//!
//! Rust deliberately does not call `Drop` for `static` values when a program
//! exits, which applies to every flavor of lazy static — `lazy_static!`,
//! `OnceLock`, `once_cell::Lazy`, `LazyLock`, etc. For most types the lack of
//! cleanup is harmless because the OS reclaims the process's memory, but
//! `Drop` impls with observable side effects (deleting a `TempDir`, killing a
//! child process, flushing a file) get silently skipped.
//!
//! [`StaticDrop<T>`] is a thin wrapper that registers each instance in a
//! process-global registry on construction. A [`dtor::dtor`] runs at program
//! exit and invokes `T::drop` on every entry still in the registry — i.e.
//! everything stored in a `static`. Values dropped normally (locals, fields,
//! etc.) deregister themselves first so they aren't dropped twice.
//!
//! ```
//! # use cryfs_utils::testutils::static_drop::StaticDrop;
//! use lazy_static::lazy_static;
//! use tempfile::TempDir;
//!
//! lazy_static! {
//!     // Without `StaticDrop`, the `TempDir`'s `Drop` would never run and
//!     // the temporary folder would leak into `$TMPDIR` on every program
//!     // exit. With it, the folder is removed at exit.
//!     static ref TMP: StaticDrop<TempDir> = StaticDrop::new(
//!         tempfile::tempdir().unwrap()
//!     );
//! }
//! ```
//!
//! # Caveats
//!
//! - **Sync `Drop` only.** The exit-time destructor cannot `.await`. Async
//!   cleanup needs a different solution (the cryfs `AsyncDropGuard` pattern
//!   doesn't fit into a `static` either).
//! - **Soft exits only.** The destructor is registered via `libc::atexit`,
//!   so it runs on both normal `main` return and [`std::process::exit`]
//!   (including the `process::exit(101)` that `libtest` uses on test
//!   failure). It does **not** run on [`std::process::abort`],
//!   `libc::_exit`, unhandled SIGTERM/SIGINT/SIGKILL, or an aborted
//!   panic — those bypass `atexit` entirely. For signal coverage see
//!   `crate::at_exit`, but mixing it with `StaticDrop` introduces races
//!   with the running program and is not done by default.
//! - **Threads may still be live.** No ordering guarantees vs. other threads
//!   at exit; treat `Drop` here the same way you would a C++ exit-time
//!   destructor — keep it self-contained.
//! - **`StaticDrop` is not meant for production code.** Prefer owning
//!   cleanup-requiring resources from `main` (or the test/runner entry
//!   point) and passing them down. This wrapper is for cases where that's
//!   genuinely awkward, such as `lazy_static!`-shared fixtures in
//!   integration tests.

use std::mem::ManuallyDrop;
use std::ops::Deref;
use std::ptr;
use std::sync::Mutex;

/// A type-erased pointer + drop function. The pointer is to a heap allocation
/// owned by the corresponding `StaticDrop<T>`, so it stays valid for the
/// program lifetime once registered.
struct Entry {
    ptr: *mut (),
    drop_fn: unsafe fn(*mut ()),
}

// Safety: entries are only ever touched while holding `REGISTRY`'s `Mutex`,
// so multiple threads cannot race on the raw pointer. The pointer is only
// dereferenced inside the exit-time destructor, by which point only the
// destructor thread runs.
unsafe impl Send for Entry {}

static REGISTRY: Mutex<Vec<Entry>> = Mutex::new(Vec::new());

/// Wrapper that runs `T`'s `Drop` at process exit even when stored in a
/// `static`. See [the module docs](self) for details and caveats.
pub struct StaticDrop<T> {
    // `Box<T>` is load-bearing: it gives the inner `T` a stable heap
    // address even if the outer `StaticDrop` is moved between
    // construction and reaching its final storage location, which keeps
    // the registered pointer valid.
    //
    // `ManuallyDrop` is needed because we drop the inner explicitly from
    // both the normal `Drop` impl (deregister, then drop) and the
    // process-exit destructor (no deregister needed).
    inner: ManuallyDrop<Box<T>>,
}

impl<T> StaticDrop<T> {
    /// Wrap `value` and register it for cleanup at process exit.
    pub fn new(value: T) -> Self {
        let inner = ManuallyDrop::new(Box::new(value));
        let ptr: *mut T = &**inner as *const T as *mut T;
        REGISTRY.lock().unwrap().push(Entry {
            ptr: ptr.cast(),
            drop_fn: |p| unsafe { ptr::drop_in_place(p.cast::<T>()) },
        });
        Self { inner }
    }
}

impl<T> Deref for StaticDrop<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.inner
    }
}

impl<T> Drop for StaticDrop<T> {
    fn drop(&mut self) {
        // Deregister so the process-exit destructor doesn't double-drop
        // this value. (Statics never reach this branch — that's the whole
        // point.)
        let ptr: *mut T = &mut **self.inner as *mut T;
        let mut reg = REGISTRY.lock().unwrap();
        if let Some(pos) = reg.iter().position(|e| e.ptr == ptr.cast()) {
            reg.swap_remove(pos);
        }
        drop(reg);
        // Run the inner `Drop` and free the `Box` allocation.
        // Safety: `ManuallyDrop::drop` is called exactly once, here.
        unsafe { ManuallyDrop::drop(&mut self.inner) };
    }
}

#[dtor::dtor(unsafe, method = at_binary_exit)]
fn cleanup_leaked_statics() {
    // Walk the registry and drop everything still in it — i.e. everything
    // stored in a `static`. `catch_unwind` so a single panicking `Drop`
    // doesn't abort the rest of the cleanup.
    //
    // `method = "at_binary_exit"` registers via `libc::atexit` rather than
    // the platform's `.fini_array`-equivalent, which means this also runs
    // on `std::process::exit(N)` (including when `libtest` exits with 101
    // after a test failure). It still does NOT run on `abort`, `_exit`,
    // or signal kill — see the module docs.
    let entries = std::mem::take(&mut *REGISTRY.lock().unwrap());
    for entry in entries {
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
            (entry.drop_fn)(entry.ptr)
        }));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// A type whose `Drop` increments a shared counter, so tests can observe
    /// whether it ran.
    struct DropCounter {
        counter: Arc<AtomicUsize>,
    }

    impl Drop for DropCounter {
        fn drop(&mut self) {
            self.counter.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[test]
    fn drop_runs_when_wrapper_goes_out_of_scope() {
        let counter = Arc::new(AtomicUsize::new(0));
        {
            let _wrap = StaticDrop::new(DropCounter {
                counter: counter.clone(),
            });
            assert_eq!(counter.load(Ordering::SeqCst), 0);
        }
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn deref_exposes_inner_value() {
        let wrap = StaticDrop::new(42u32);
        assert_eq!(*wrap, 42);
    }

    #[test]
    fn deref_chain_through_lazy_static_wrapper() {
        // Reproduces the access pattern used by callers: a lazy-static
        // wrapper holding a `StaticDrop<T>` deref-chains all the way to
        // `&T` for both field/method calls and `&value` coercions.
        struct Inner {
            value: u32,
        }
        impl Inner {
            fn double(&self) -> u32 {
                self.value * 2
            }
        }
        static X: std::sync::LazyLock<StaticDrop<Inner>> =
            std::sync::LazyLock::new(|| StaticDrop::new(Inner { value: 21 }));
        // Method call via double deref (LazyLock → StaticDrop → Inner).
        assert_eq!(X.double(), 42);
        // Field access via double deref.
        assert_eq!(X.value, 21);
        // `&*X` coerces to `&Inner` via deref-coercion through both layers.
        fn takes_inner(i: &Inner) -> u32 {
            i.value
        }
        assert_eq!(takes_inner(&X), 21);
    }

    #[test]
    fn moving_the_wrapper_does_not_invalidate_registration() {
        // The registry holds a pointer to the heap allocation owned by the
        // wrapper. Moving the wrapper itself must not invalidate that
        // pointer, otherwise the exit-time destructor would dereference
        // freed memory.
        let counter = Arc::new(AtomicUsize::new(0));
        let outer;
        {
            let inner = StaticDrop::new(DropCounter {
                counter: counter.clone(),
            });
            outer = inner; // move
        }
        // Still not dropped — it lives in `outer`.
        assert_eq!(counter.load(Ordering::SeqCst), 0);
        drop(outer);
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn dropping_does_not_affect_other_registered_values() {
        // Verify that `Drop`'s deregister-by-pointer-match doesn't
        // accidentally remove someone else's entry.
        let counter_a = Arc::new(AtomicUsize::new(0));
        let counter_b = Arc::new(AtomicUsize::new(0));
        let a = StaticDrop::new(DropCounter {
            counter: counter_a.clone(),
        });
        let b = StaticDrop::new(DropCounter {
            counter: counter_b.clone(),
        });
        drop(a);
        assert_eq!(counter_a.load(Ordering::SeqCst), 1);
        assert_eq!(counter_b.load(Ordering::SeqCst), 0);
        drop(b);
        assert_eq!(counter_b.load(Ordering::SeqCst), 1);
    }

    /// Compile-time check: `StaticDrop<T>: Sync` (required to store it in a
    /// `static`) and `Send` when `T` is. A future field addition that
    /// introduces e.g. a `Cell` would silently break this, so pin it here.
    const _: () = {
        const fn assert_send<T: Send>() {}
        const fn assert_sync<T: Sync>() {}
        assert_send::<StaticDrop<u32>>();
        assert_sync::<StaticDrop<u32>>();
        // String is `Send + Sync`; covers a non-`Copy` payload.
        assert_send::<StaticDrop<String>>();
        assert_sync::<StaticDrop<String>>();
    };

    #[test]
    fn drop_does_not_run_twice() {
        // The normal `Drop` deregisters, so the exit-time destructor must
        // not re-drop. We can't easily observe the exit destructor from a
        // unit test, but we can at least verify that the normal `Drop`
        // runs exactly once.
        let counter = Arc::new(AtomicUsize::new(0));
        let wrap = StaticDrop::new(DropCounter {
            counter: counter.clone(),
        });
        drop(wrap);
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }
}
