/// A macro that panics but avoids double-panics because they don't show useful error messages.
/// If the thread is already panicking, it prints an error message to stderr instead.
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

// TODO Tests
