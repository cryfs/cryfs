//! Safe panic macro that avoids double-panics.
//!
//! This module provides the [`safe_panic!`] macro, which behaves like [`panic!`]
//! but avoids double-panics by printing to stderr instead if the thread is already
//! panicking. Double-panics typically don't show useful error messages, so this
//! macro provides better debugging information in those scenarios.

/// A macro that panics but avoids double-panics because they don't show useful error messages.
///
/// If the thread is already panicking, it prints an error message to stderr instead
/// of causing a double-panic. This is useful in destructors or cleanup code where
/// a panic might occur during an existing panic unwind.
///
/// # Examples
///
/// ```should_panic
/// use cryfs_utils::safe_panic;
///
/// // Normal usage - panics like regular panic!
/// safe_panic!("Something went wrong");
/// ```
///
/// ```
/// use cryfs_utils::safe_panic;
///
/// // During a panic, it prints to stderr instead of double-panicking
/// struct PanicOnDrop;
/// impl Drop for PanicOnDrop {
///     fn drop(&mut self) {
///         safe_panic!("This would be a double-panic");
///     }
/// }
///
/// // This won't cause a double-panic abort
/// let result = std::panic::catch_unwind(|| {
///     let _guard = PanicOnDrop;
///     panic!("First panic");
/// });
/// assert!(result.is_err());
/// ```
#[macro_export]
macro_rules! safe_panic {
    ($($arg:tt)*) => {
        if std::thread::panicking() {
            // We're already panicking, double panic wouldn't show a good error message anyways. Let's just log instead.
            // A common scenario for this to happen is a failing test case.
            eprint!("Panic while already panicking: ");
            eprintln!($($arg)*);
        } else {
            panic!($($arg)*);
        }
    };
}

#[cfg(test)]
mod tests {
    #[test]
    #[should_panic(expected = "test panic message")]
    fn test_safe_panic_panics_normally() {
        safe_panic!("test panic message");
    }

    #[test]
    #[should_panic(expected = "formatted: 42")]
    fn test_safe_panic_with_formatting() {
        safe_panic!("formatted: {}", 42);
    }

    #[test]
    fn test_safe_panic_during_panic_does_not_abort() {
        // Create a type that calls safe_panic! in its destructor
        struct SafePanicOnDrop;
        impl Drop for SafePanicOnDrop {
            fn drop(&mut self) {
                // This would cause a double-panic with regular panic!
                // but safe_panic! just prints to stderr instead
                safe_panic!("safe panic in drop");
            }
        }

        // Catch the first panic - the safe_panic in drop should NOT abort
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = SafePanicOnDrop;
            panic!("first panic");
        }));

        // The panic should have been caught (not aborted due to double-panic)
        assert!(result.is_err());
    }
}
