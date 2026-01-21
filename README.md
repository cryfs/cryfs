# CryFS 2.0 (Alpha)

## ⚠️ ALPHA VERSION WARNING ⚠️

**CryFS 2.0-alpha is experimental software. You WILL lose your data. Do not use it for anything important.**

For stable CryFS, please use [CryFS 1.0](https://github.com/cryfs/cryfs/tree/release/1.0). You can find installation instructions for the stable version [here](https://www.cryfs.org/#download).

CryFS 2.0 is in an alpha stage that has not undergone extensive testing. Use it only for testing purposes and please report any issues in the [GitHub issue tracker](https://github.com/cryfs/cryfs/issues).

---

## What is CryFS?

CryFS encrypts your files so you can safely store them in cloud storage services like Dropbox, iCloud, or OneDrive. Unlike other encryption tools that encrypt files individually, CryFS encrypts your files in a way that also hides file sizes, directory structure, and metadata, providing a higher level of security.

Learn more at [https://www.cryfs.org](https://www.cryfs.org)

## What's New in CryFS 2.0?

CryFS 2.0 is a complete rewrite from scratch in Rust, bringing improved memory safety. This version is currently in alpha and represents a major evolution of the project.

### Key Changes

To ensure compatiblity with CryFS 1.0, CryFS 2.0 does not add any features that would make the filesystem incompatible with Cryfs 1.0. New breaking changes are planned for future versions after CryFS 2.0 is stable. However, there are a few minor differences between CryFS 1.0 and 2.0.

**New Features:**
- Configurable scrypt parameters when creating new file systems
- Updated default scrypt parameters for better security
- Human-readable duration format for `--unmount-idle` (e.g., `5m`, `1h30m`)
- More explicit units for `--blocksize` (e.g., `16KiB`)
- More flexible logging with `--log` argument (e.g., `--log file:/path/to/file.log`)

**Breaking Changes:**
- Command line options have changed (see [Command Line Changes](#command-line-changes) below)
- Limited cipher support: only XChaCha20 and AES are available
- Reduced platform support: only Linux is currently supported

## Platform Support

| Platform | Status |
|----------|--------|
| Linux    | ✅ Working |
| macOS    | ❓ Untested (may or may not work) |
| Windows  | ❌ Not yet supported |

## Compatibility with CryFS 1.0

### Filesystem Compatibility

File systems are **fully forward and backward compatible** between CryFS 1.0 and 2.0, with important caveats:

✅ **Compatible:**
- File systems created with XChaCha20 cipher (the default in 1.0 and 2.0)
- File systems created with AES-256-GCM cipher
- Integrity checks using block versioning are fully compatible

⚠️ **Partially Compatible:**
- Local state files: The filesystem ID verification (protection against filesystem replacement attacks) uses separate local state files in 1.0 vs 2.0. Both versions perform this check, but they don't sync with each other.

❌ **Incompatible:**
- File systems created with other ciphers (e.g., Twofish, Serpent) are not accessible in CryFS 2.0
- There are no plans to add all ciphers from CryFS 1.0 to the Rust version because many are outdated and don't have an implementaton that can be called from Rust easily.

### Command Line Changes

The following command line arguments have changed:

| 1.0 Syntax | 2.0 Syntax | Notes |
|------------|------------|-------|
| `--unmount-idle 10` | `--unmount-idle 10m` | Now requires unit: `5m`, `1h30m`, etc. |
| `--blocksize 16384` | `--blocksize 16KiB` | Now requires unit: `16KiB`, `1MiB`, etc. |
| `--logfile /path/to/file.log` | `--log file:/path/to/file.log` | More generic logging format |
| `cryfs vaultdir mountdir -- -o allow_other` | `cryfs vaultdir mountdir -o allow_other` | Double-dash syntax removed |

The list of supported FUSE options that can be passed in with `-o` is now limited to options that are known to work well with CryFS. See `cryfs --help` for the complete list.

## Installation

### Linux

#### Building from Source

**Prerequisites:**
- Rust toolchain (install from [rustup.rs](https://rustup.rs))
- Build dependencies
  - Ubuntu/Debian: `sudo apt install build-essential pkg-config libssl-dev`
  - Fedora: `sudo dnf install fuse3-devel`
  - Arch: `sudo pacman -S fuse3`

**Build and Install:**
```bash
git clone https://github.com/cryfs/cryfs
cd cryfs
cargo build --release
sudo cp target/release/cryfs /usr/local/bin/
```

### macOS

Not yet tested. May work if you have macFUSE installed, but no guarantees.

### Windows

Windows support is not yet available in CryFS 2.0.

## Usage

### Creating a New Encrypted Filesystem

```bash
cryfs /path/to/encrypted/storage /path/to/mountpoint
```

You'll be prompted to create a password. CryFS will create its encrypted storage in the first directory and mount the decrypted filesystem at the mountpoint.

### Mounting an Existing Filesystem

Use the same command:

```bash
cryfs /path/to/encrypted/storage /path/to/mountpoint
```

You'll be prompted for your password.

### Unmounting

```bash
fusermount -u /path/to/mountpoint
```

Or on macOS:
```bash
umount /path/to/mountpoint
```

### Advanced Options

```bash
# Auto-unmount after 30 minutes of inactivity
cryfs /path/to/encrypted /path/to/mount --unmount-idle 30m

# Allow other users to access the filesystem
cryfs /path/to/encrypted /path/to/mount -o allow_other

# Log to a file
cryfs /path/to/encrypted /path/to/mount --log file:/tmp/cryfs.log

# Show all available options
cryfs --help
```

## Graphical User Interfaces

CryFS can be used through GUI applications:
- [SiriKali](https://mhogomchungu.github.io/sirikali/)
- Plasma Vault (included in KDE Plasma 5.11+)

Note: GUI compatibility with CryFS 2.0 has not been tested yet.

## Known Issues

### Performance
CryFS 2.0-alpha is currently **slower than 1.0** due to lack of optimizations. Performance improvements are planned for future releases.

### Stability
As alpha software, expect bugs and potential data loss. Known risks include:
- Filesystem corruption if the process is interrupted during writes
- Data loss if the disk runs out of space during write operations
- Corruption if the filesystem is accessed from multiple devices simultaneously without proper synchronization

### Recovery
There is currently no filesystem recovery tool for corrupted CryFS filesystems. **Back up your data regularly.**

## Security Notes

### Password Changes
If your password is compromised, creating a new filesystem and migrating your data is strongly recommended, as CryFS does not support secure password rotation.

### Cipher Selection
CryFS 2.0 supports:
- **XChaCha20-Poly1305** (default, recommended)
- **AES-256-GCM**

XChaCha20 is the recommended cipher for new filesystems due to its strong security properties and performance characteristics.

### Scrypt Parameters
CryFS 2.0 allows you to configure scrypt parameters when creating a new filesystem, allowing you to adjust the time/memory tradeoffs for password derivation based on your security needs.

Larger parameters are more secure but mean the filesystem will be slower to mount, and devices with low memory might not be able to open it at all. There is no performance impact on filesystem operations after it was mounted, only the initial mounting is affected.

If you want to use a filesystem from devices with very low memory, lowering the scrypt parameters can help.

## Contributing

Contributions are welcome! Please:
1. Check the [issue tracker](https://github.com/cryfs/cryfs/issues) for known bugs and feature requests
2. Test CryFS 2.0 and report any bugs you find
3. Submit pull requests with improvements

Since this is alpha software, testing and bug reports are especially valuable.

## License

CryFS is licensed under the LGPL v3. See the LICENSE file for details.

## Links

- Website: [https://www.cryfs.org](https://www.cryfs.org)
- GitHub: [https://github.com/cryfs/cryfs](https://github.com/cryfs/cryfs)
- Issue Tracker: [https://github.com/cryfs/cryfs/issues](https://github.com/cryfs/cryfs/issues)
- CryFS 1.0 (stable): [https://github.com/cryfs/cryfs/tree/release/1.0](https://github.com/cryfs/cryfs/tree/release/1.0)
