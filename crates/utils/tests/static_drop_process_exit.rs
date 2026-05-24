//! Integration tests for `StaticDrop` behavior at process exit. Each test
//! has two roles selected by env var: the parent spawns itself as a child
//! with the env var set, the child sets up `StaticDrop` instances and
//! exits via `process::exit`, the parent observes side effects (sentinel
//! files in a parent-owned tempdir) after the child terminates.
//!
//! What these cover that the unit tests don't:
//!
//! 1. The exit-time destructor actually fires on `process::exit` (the
//!    `method = at_binary_exit` knob on Linux).
//! 2. Multiple `StaticDrop` entries are all dropped at exit, not just the
//!    first or last.
//! 3. A panic in one entry's `Drop` doesn't prevent the others from being
//!    dropped (the `catch_unwind` around each `drop_fn` call).
//!
//! Requires the `testutils` feature — without it, `cryfs_utils::testutils`
//! is `cfg`-ed out and this file compiles to an empty test binary.
#![cfg(feature = "testutils")]

use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::LazyLock;

use cryfs_utils::testutils::static_drop::StaticDrop;
use tempfile::TempDir;

/// Set in the child invocation so the test function knows which role to
/// play.
const CHILD_ROLE_ENV_VAR: &str = "STATIC_DROP_PROCESS_EXIT_CHILD";
/// Path of the parent-owned tempdir, passed to the child via env so it
/// can write sentinel files the parent will inspect.
const CHILD_SENTINEL_DIR_ENV_VAR: &str = "STATIC_DROP_SENTINEL_DIR";

fn is_child() -> bool {
    env::var(CHILD_ROLE_ENV_VAR).is_ok()
}

fn sentinel_dir() -> PathBuf {
    env::var(CHILD_SENTINEL_DIR_ENV_VAR)
        .expect("CHILD_SENTINEL_DIR_ENV_VAR not set in child")
        .into()
}

/// Spawn the same test binary as a child, filtered to just `test_name`,
/// with the child role + sentinel dir env vars set.
fn spawn_child(test_name: &str, sentinel_dir: &Path) -> Output {
    let me = env::current_exe().expect("current_exe");
    Command::new(&me)
        .arg("--exact")
        .arg(test_name)
        .arg("--nocapture")
        .env(CHILD_ROLE_ENV_VAR, "1")
        .env(CHILD_SENTINEL_DIR_ENV_VAR, sentinel_dir)
        .output()
        .expect("spawn child")
}

fn assert_child_succeeded(out: &Output) {
    assert!(
        out.status.success(),
        "child did not exit cleanly: status={:?}\nstdout:\n{}\nstderr:\n{}",
        out.status,
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );
}

// ---------------------------------------------------------------------------
// Test 1: single StaticDrop cleaned up at process::exit
// ---------------------------------------------------------------------------

#[test]
fn static_drop_runs_on_process_exit() {
    if is_child() {
        run_single_tempdir_child();
    }

    let parent_dir = tempfile::tempdir().expect("parent tempdir");
    let out = spawn_child("static_drop_runs_on_process_exit", parent_dir.path());
    assert_child_succeeded(&out);

    let stdout = String::from_utf8(out.stdout).expect("child stdout utf8");
    let tempdir_path = stdout
        .lines()
        .find_map(|l| l.strip_prefix("STATIC_DROP_TEMPDIR="))
        .expect("child did not print tempdir path");
    let tempdir_path = Path::new(tempdir_path);

    assert!(
        !tempdir_path.exists(),
        "tempdir was not removed at process::exit, so the `StaticDrop` \
         destructor did not fire: {}",
        tempdir_path.display()
    );
}

fn run_single_tempdir_child() -> ! {
    static TMP: LazyLock<StaticDrop<TempDir>> =
        LazyLock::new(|| StaticDrop::new(tempfile::tempdir().unwrap()));
    let path = TMP.path().to_owned();
    println!("STATIC_DROP_TEMPDIR={}", path.display());
    assert!(path.exists(), "tempdir does not exist before process::exit");
    // Exit via `process::exit` rather than returning so the destructor
    // path under test (`method = at_binary_exit`) is actually exercised.
    std::process::exit(0);
}

// ---------------------------------------------------------------------------
// Test 2: multiple StaticDrop entries all cleaned up at process::exit
// ---------------------------------------------------------------------------

/// Writes a sentinel file on `Drop`. We use file-existence as a witness
/// because the dtor runs in a child process — the parent can't observe
/// the child's stdout after `exit`, but it can observe the filesystem.
struct DropSentinel {
    sentinel: PathBuf,
}

impl Drop for DropSentinel {
    fn drop(&mut self) {
        std::fs::write(&self.sentinel, "dropped").expect("write sentinel");
    }
}

#[test]
fn static_drop_runs_for_every_registered_entry() {
    if is_child() {
        run_multiple_entries_child();
    }

    let parent_dir = tempfile::tempdir().expect("parent tempdir");
    let out = spawn_child(
        "static_drop_runs_for_every_registered_entry",
        parent_dir.path(),
    );
    assert_child_succeeded(&out);

    for name in ["a", "b", "c"] {
        let sentinel = parent_dir.path().join(name);
        assert!(
            sentinel.exists(),
            "sentinel for entry {name} was not written — its `Drop` did not \
             run at process::exit. Missing: {}",
            sentinel.display(),
        );
    }
}

fn run_multiple_entries_child() -> ! {
    static A: LazyLock<StaticDrop<DropSentinel>> = LazyLock::new(|| {
        StaticDrop::new(DropSentinel {
            sentinel: sentinel_dir().join("a"),
        })
    });
    static B: LazyLock<StaticDrop<DropSentinel>> = LazyLock::new(|| {
        StaticDrop::new(DropSentinel {
            sentinel: sentinel_dir().join("b"),
        })
    });
    static C: LazyLock<StaticDrop<DropSentinel>> = LazyLock::new(|| {
        StaticDrop::new(DropSentinel {
            sentinel: sentinel_dir().join("c"),
        })
    });
    // Force lazy init of all three so they're registered.
    let _ = (&**A, &**B, &**C);
    std::process::exit(0);
}

// ---------------------------------------------------------------------------
// Test 3: a panicking Drop doesn't prevent siblings from being dropped
// ---------------------------------------------------------------------------

/// Like `DropSentinel`, but panics after writing the sentinel. The
/// `catch_unwind` around each entry's `drop_fn` should swallow this and
/// continue with the remaining entries.
struct PanicAfterWritingSentinel {
    sentinel: PathBuf,
}

impl Drop for PanicAfterWritingSentinel {
    fn drop(&mut self) {
        std::fs::write(&self.sentinel, "dropped").expect("write sentinel");
        panic!("intentional panic in Drop for the test harness");
    }
}

#[test]
fn static_drop_continues_after_panic_in_one_drop() {
    if is_child() {
        run_panic_in_middle_child();
    }

    let parent_dir = tempfile::tempdir().expect("parent tempdir");
    let out = spawn_child(
        "static_drop_continues_after_panic_in_one_drop",
        parent_dir.path(),
    );
    // The child must still exit cleanly — `catch_unwind` in the dtor
    // swallows the panic so it doesn't reach the runtime.
    assert_child_succeeded(&out);

    for name in ["a", "b", "c"] {
        let sentinel = parent_dir.path().join(name);
        assert!(
            sentinel.exists(),
            "sentinel for entry {name} missing — the panic in b's Drop \
             aborted cleanup of the other entries. Missing: {}",
            sentinel.display(),
        );
    }
}

fn run_panic_in_middle_child() -> ! {
    static A: LazyLock<StaticDrop<DropSentinel>> = LazyLock::new(|| {
        StaticDrop::new(DropSentinel {
            sentinel: sentinel_dir().join("a"),
        })
    });
    static B: LazyLock<StaticDrop<PanicAfterWritingSentinel>> = LazyLock::new(|| {
        StaticDrop::new(PanicAfterWritingSentinel {
            sentinel: sentinel_dir().join("b"),
        })
    });
    static C: LazyLock<StaticDrop<DropSentinel>> = LazyLock::new(|| {
        StaticDrop::new(DropSentinel {
            sentinel: sentinel_dir().join("c"),
        })
    });
    let _ = (&**A, &**B, &**C);
    std::process::exit(0);
}
