//! Verifies that `StaticDrop`'s registered destructor actually fires on
//! `std::process::exit(N)`, not just on a normal `main` return. On Linux
//! the default `dtor` registration uses `.fini_array`, which is skipped by
//! `process::exit`; this test would fail without the `method = at_binary_exit`
//! override in `static_drop.rs`.
//!
//! Both the parent and child role run inside the same integration-test
//! binary; the child role is selected by setting a sentinel env var.
//!
//! Requires the `testutils` feature — without it, `cryfs_utils::testutils`
//! is `cfg`-ed out and this file compiles to an empty test binary.
#![cfg(feature = "testutils")]

use std::env;
use std::path::Path;
use std::process::Command;
use std::sync::LazyLock;

use cryfs_utils::testutils::static_drop::StaticDrop;
use tempfile::TempDir;

const CHILD_ROLE_ENV_VAR: &str = "STATIC_DROP_PROCESS_EXIT_CHILD";
const TEST_NAME: &str = "static_drop_runs_on_process_exit";

static TMP: LazyLock<StaticDrop<TempDir>> =
    LazyLock::new(|| StaticDrop::new(tempfile::tempdir().unwrap()));

#[test]
fn static_drop_runs_on_process_exit() {
    if env::var(CHILD_ROLE_ENV_VAR).is_ok() {
        run_child_role();
        unreachable!("run_child_role should have called process::exit");
    }

    // Parent role: spawn self with the child env var set, filtered to just
    // this test so libtest only runs `run_child_role`.
    let me = env::current_exe().expect("current_exe");
    let output = Command::new(&me)
        .arg("--exact")
        .arg(TEST_NAME)
        .arg("--nocapture")
        .env(CHILD_ROLE_ENV_VAR, "1")
        .output()
        .expect("spawn child");

    assert!(
        output.status.success() || output.status.code() == Some(0),
        "child did not exit cleanly: status={:?}, stderr={}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("child stdout utf8");
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

fn run_child_role() {
    // Force lazy init, then publish the path so the parent can check it.
    let path = TMP.path().to_owned();
    println!("STATIC_DROP_TEMPDIR={}", path.display());
    // Sanity: the directory must exist right now. If this fails, the rest
    // of the assertion is meaningless.
    assert!(
        path.exists(),
        "child: tempdir does not exist before process::exit"
    );

    // Exit via `process::exit` specifically — not by returning from this
    // function, which would trigger the test harness's normal-exit path
    // and could mask a regression in `at_binary_exit`.
    std::process::exit(0);
}
