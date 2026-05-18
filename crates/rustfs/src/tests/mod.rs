// TODO Our tests currently only work for the fuser backend, we should add the fuse-mt backend as well.
#![cfg(feature = "fuser")]
// These tests mount real FUSE sessions via fuser. They run fine on a
// local macOS where the operator has approved the macFUSE kernel
// extension, but on GitHub-hosted macOS runners the kext can't load
// (https://github.com/actions/runner-images/issues/4731) and the
// mount() syscall hangs. We deliberately do NOT gate on
// cfg(target_os = "macos") so the tests stay runnable locally; the
// CI-only skip lives in .github/workflows/ci.yml.

mod utils;

mod mkdir;
