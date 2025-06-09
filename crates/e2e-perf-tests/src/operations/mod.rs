pub mod chmod;
pub mod chown;
pub mod create_file;
pub mod fchmod;
pub mod fchown;
pub mod fgetattr;
pub mod fsync;
pub mod ftruncate;
pub mod futimens;
pub mod getattr;
pub mod init;
pub mod lookup;
pub mod mkdir;
pub mod open;
pub mod read;
pub mod readdir;
pub mod readlink;
pub mod release;
pub mod rename;
pub mod rmdir;
pub mod statfs;
pub mod symlink;
pub mod truncate;
pub mod unlink;
pub mod utimens;
pub mod write;

// TODO It would be nice to split all counts into (1) operation itself and (2) the automatic flush we do right after, to test that caching works.
//     Or to do the same for benchmarks, maybe split it into two tests, one with and one without that flushing?
// TODO Somehow none of the operations have different counts based on the atime update behavior? That seems odd, shouldn't the atime update behavior affect the number of operations needed?
//      Operations that should be affected: mkdir, create_file, symlink (they need to update the parent dir's timestamp in the grandparent dir), readdir, rename, rmdir, read, write, unlink, others? readdir and readlink do change based on atime somehow. But correctly?
// TODO Go through all the operation counts in the tests and think about whether we can reduce the number of needed operations
// TODO Would be nice to expand this crate to also test correctness of the operations, e.g. add a .expect_output() function to test driver in addition to .expect_op_count()
// TODO Find cases where .setup() passes data to .test() that isn't actually used in the test function, and remove it.
// TODO Find cases of magic strings (e.g. paths, filenames) that are repeated and put them into constants
// TODO For benchmarks, it might make sense to increase the block size to real world block sizes (e.g. 16kb or 32kb)
// TODO The benchmarks, e.g. `cargo bench --features benchmarking symlink` sometimes seem to get stuck (deadlock?) but always in the first benchmark to be executed, if that passes, then later benchmarks in the same run are fine. Maybe deadlock in setup code?
