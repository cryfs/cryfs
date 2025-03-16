TODO Write README for rust version

[todo] add link to stable version README

Known differences between 1.0 and 2.0-alpha:
- File systems should be fully forward and backward compatible, if using one of the ciphers supported in 2.0-alpha.
- Not all ciphers supported in 1.0 are available in 2.0-alpha
- 2.0-alpha is currently slower than 1.0 (due to lack of optimizations)
- Breaking change in `--unmount-idle` argument. It doesn't take number of minutes anymore but allows specifying a human readable duration (e.g. 5m, 1h30m)
- Breaking change in `--blocksize` argument. It now requires specifying a unit (e.g. '16KiB') instead of just a number of bytes.
- Breaking change: The `--logfile` argument was replaced with a more generic `--log` argument. To log to a file, replace `--logfile /path/to/file.log` with `--log file:/path/to/file.log`.
- Local state is only partially migrated between 1.0 and 2.0. Known block versions are fully forward and backward compatible, but the list of filesystem ids per basedir (i.e. the check that an attacker didn't replace the whole filesystem with a different filesystem) will not sync between 1.0 and 2.0. Both versions do have the check, but use their own version of the local state.
- Windows and Mac don't work yet, only Linux for now
- The old style of passing fuse options behind a double dash (e.g. `cryfs basedir mountdir -- allow_other`) was already deprecated in 1.0 and is now removed. Please use `cryfs basedir mountdir -o allow_other` instead.
  Also, the list of supported options is now limited to options that are known to work well with CryFS. See `cryfs --help` in CryFS 2.0 for the list of supported options.

Some other changes:
* scrypt parameters are now configurable when creating a new file system.
* updated default scrypt parameters
