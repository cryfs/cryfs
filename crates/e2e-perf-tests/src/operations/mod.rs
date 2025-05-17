mod chmod;
mod chown;
mod create_file;
mod fchmod;
mod fchown;
mod fgetattr;
mod flush;
mod fsync;
mod ftruncate;
mod futimens;
mod getattr;
mod init;
mod lookup;
mod mkdir;
mod open;
mod read;
mod readdir;
mod readlink;
mod rename;
mod rmdir;
mod statfs;
mod symlink;
mod truncate;
mod unlink;
mod utimens;
mod write;

// TODO It would be nice to split all counts into (1) operation itself and (2) the automatic flush we do right after, to test that caching works.
// TODO Somehow none of the operations have different counts based on the atime update behavior? That seems odd, shouldn't the atime update behavior affect the number of operations needed?
//      Operations that should be affected: mkdir, create_file, symlink (they need to update the parent dir's timestamp in the grandparent dir), readdir, rename, rmdir, read, write, unlink, others? readdir and readlink do change based on atime somehow. But correctly?
// TODO Test other operations: flush, fsync
// TODO Go through all the operation counts in the tests and think about whether we can reduce the number of needed operations
