mod create_file;
mod fgetattr;
mod getattr;
mod init;
mod lookup;
mod mkdir;
mod open;
mod readdir;
mod readlink;
mod statfs;
mod symlink;

// TODO Somehow none of the operations have different counts based on the atime update behavior? That seems odd, shouldn't the atime update behavior affect the number of operations needed?
//      Operations that should be affected: mkdir, create_file, symlink (they need to update the parent dir's timestamp in the grandparent dir), readdir, rename, rmdir, read, write, unlink, others? readdir and readlink do change based on atime somehow. But correctly?
// TODO Test other operations: setattr/fsetattr (chmod/fchmod, chown/fchown, chgrp/fchgrp, truncate/ftruncate, utimens, futimens), unlink, rmdir, rename, read, write, flush, fsync
// TODO Go through all the operation counts in the tests and think about whether we can reduce the number of needed operations
