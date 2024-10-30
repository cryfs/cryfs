TODO Write README for rust version

[todo] add link to stable version README

Known differences between 1.0 and 2.0-alpha:
- File systems should be fully forward and backward compatible, if using one of the ciphers supported in 2.0-alpha.
- Not all ciphers supported in 1.0 are available in 2.0-alpha
- 2.0-alpha is currently slower than 1.0 (due to lack of optimizations)
- Breaking change in `--unmount-idle` argument. It doesn't take number of minutes anymore but allows specifying a human readable duration (e.g. 5m, 1h30m)
- Windows and Mac don't work yet, only Linux for now
