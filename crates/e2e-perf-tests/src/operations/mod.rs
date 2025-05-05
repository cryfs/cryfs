mod create_file;
mod fgetattr;
mod getattr;
mod init;
mod lookup;
mod mkdir;

// TODO Somehow none of the operations have different counts based on the atime update behavior? That seems odd, shouldn't the atime update behavior affect the number of operations needed?
// TODO Test other operations: setattr/fsetattr (chmod/fchmod, chown/fchown, chgrp/fchgrp, truncate/ftruncate, utimens, futimens), readlink, unlink, rmdir, symlink, rename, open, read, write, flush, fsync, readdir, statfs
// TODO Go through all the operation counts in the tests and think about whether we can reduce the number of needed operations
