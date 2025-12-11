TODO Write README for rust version

[todo] add link to stable version README

Known differences between 1.0 and 2.0-alpha:
- Expect bugs in 2.0-alpha. It hasn't gone through any extensive testing yet. You WILL lose your data. Please test and report any issues in the GitHub issue tracker.
- File systems should be fully forward and backward compatible, if using one of the ciphers supported in 2.0-alpha (see next point).
- Not all ciphers supported in 1.0 are available in 2.0-alpha. Cryfs 1.0 supported a large selection of ciphers, with XChaCha20 being the default. Cryfs 2.0 only supports XChaCha20 and AES.
  All Cryfs 1.0 file systems created with the default cipher will load correctly, but if you used one of the other ciphers, it will not be accessble by Cryfs 2.0.
  This will likely stay this way, there are no plans to add all ciphers from Cryfs 1.0 to the rust version.
- 2.0-alpha is currently slower than 1.0 (due to lack of optimizations)
- Local state is only partially migrated between 1.0 and 2.0. Known block versions are fully forward and backward compatible, but the list of filesystem ids per basedir (i.e. the check that an attacker didn't replace the whole filesystem with a different filesystem) will not sync between 1.0 and 2.0. Both versions do have the check, but track this each in their separate file locally and don't sync.
- Windows doesn't work yet, only Linux is known to work for now. MacOS may or may not work.
- Breaking command line changes
  - `--unmount-idle` doesn't take number of minutes anymore but allows specifying a human readable duration (e.g. 5m, 1h30m)
  - `--blocksize` now requires specifying a unit (e.g. '16KiB') instead of just a number of bytes.
  - `--logfile` was replaced with a more generic `--log` argument. To log to a file, replace `--logfile /path/to/file.log` with `--log file:/path/to/file.log`.
  - The old style of passing fuse options behind a double dash (e.g. `cryfs basedir mountdir -- allow_other`) was already deprecated in 1.0 and is now removed. Please use `cryfs basedir mountdir -o allow_other` instead.
    The list of supported options is now limited to options that are known to work well with CryFS. See `cryfs --help` in CryFS 2.0 for the list of supported options.

Some other changes:
* scrypt parameters are now configurable when creating a new file system.
* updated default scrypt parameters
