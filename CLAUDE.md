# CryFS Architecture Guide

This document helps AI coding agents understand the CryFS codebase structure, patterns, and conventions.

## Project Overview

CryFS is an encrypted filesystem for cloud storage (Dropbox, iCloud, OneDrive). It encrypts files while hiding file sizes, directory structure, and metadata.

- Rust workspace with several crates
- Targets newest Rust edition and version

## Architecture

The architecture is layered. Dependencies flow downward (higher layers depend on lower layers):

```
┌─────────────────────────────────────────────────────────────┐
│  APPLICATION LAYER                                          │
│  cryfs-cli ──────────► cryfs-runner                        │
│  (CLI binary)          (mount orchestration)               │
│       │                     │                              │
│       └──► cli-utils ◄──────┘                              │
│            (shared CLI code, blockstore setup)             │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│  FILESYSTEM LAYER                                           │
│  cryfs-filesystem (CryDevice - implements FUSE operations) │
│       │                                                    │
│       └──► rustfs (FUSE abstraction, Device trait)         │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│  STORAGE LAYER (bottom-up within this layer)               │
│                                                             │
│  fsblobstore  (filesystem semantics: file/dir/symlink)     │
│       │                                                     │
│       ▼                                                     │
│  blobstore    (variable-size data as trees of blocks)      │
│       │                                                     │
│       ▼                                                     │
│  blockstore   (fixed-size encrypted blocks)                │
└─────────────────────────────────────────────────────────────┘

Cross-cutting: crypto, utils, cryfs-version, concurrent-store, cryfs-config
```

## Crate Directory

### Storage Layer

| Crate | Purpose | Key Types |
|-------|---------|-----------|
| blockstore | Fixed-size encrypted blocks | BlockId, BlockStore* traits |
| blobstore | Variable-size data on blocks | Blob, BlobStore, DataTree |
| fsblobstore | Filesystem blob semantics | FsBlobStore, ConcurrentFsBlobStore |

### Filesystem Layer

| Crate | Purpose | Key Types |
|-------|---------|-----------|
| rustfs | FUSE abstraction layer | Device trait, fuser backend |
| cryfs-filesystem | FUSE filesystem implementation | CryDevice, CryFile, CryDir, CryNode |

### Application Layer

| Crate | Purpose | Key Types |
|-------|---------|-----------|
| cryfs-cli | Main CLI binary (cryfs) | argument parsing, console interaction |
| cryfs-runner | Mount orchestration | FilesystemRunner, MountArgs |
| cli-utils | Shared CLI utilities | blockstore stack setup, password prompts |

### Cross-cutting

| Crate | Purpose | Key Types |
|-------|---------|-----------|
| crypto | Cryptographic primitives | Cipher, KDF, Hash |
| cryfs-config | Configuration management | CryConfig, LocalStateDir |
| utils | Common utilities | AsyncDrop, Data, temp files |
| concurrent-store | Thread-safe caching | ConcurrentStore |
| cryfs-version | Version macros | git tag verification |

### Testing/Development

| Crate | Purpose |
|-------|---------|
| check | Filesystem integrity checker (cryfs-check binary) |
| tempproject | Test utilities |
| e2e-perf-tests | Performance benchmarks |

## BlockStore Stack

The blockstore uses a decorator pattern. Layers wrap each other (innermost to outermost):

```
OnDiskBlockStore (disk I/O)
      ↓
EncryptedBlockStore (encryption via crypto crate)
      ↓
IntegrityBlockStore (integrity verification, versioning)
      ↓
LockingBlockStore (per-block concurrency control)
```

See `crates/cli-utils/src/blockstore_setup.rs` for the setup code.

## Coding Conventions

- `#![forbid(unsafe_code)]` in most crates
- All I/O is async via tokio runtime
- Trait-based abstraction (BlockStoreReader/Writer/Deleter, Blob, Device)
- Binary serialization with `binrw` and `binary-layout`

## Code Quality Principles

- **All code should have tests** - write tests for new functionality
- **Clear architectural patterns** with low coupling between components
- **Use invariants** to reason about correctness
- **Use the type system** to enforce correctness and invariants when possible
- Prefer compile-time guarantees over runtime checks

## Error Handling Patterns

- **Library crates**: Define detailed error types with `thiserror`, NOT `anyhow`
- **CLI/application crates**: May use `anyhow` for error propagation with `.context()`
- **Calling code responsibility**: When calling library functions, check for errors and wrap/map them to your own error types where appropriate
- Note: Current codebase doesn't fully follow this yet, but new code should

## AsyncDrop Pattern

- Types needing async cleanup implement the `AsyncDrop` trait
- **Every `AsyncDropGuard<T>`** must have `async_drop()` called before being dropped (panics otherwise)
- **Factory methods** (e.g., `Self::new()`) for AsyncDrop types should return `AsyncDropGuard<Self>`, never plain `Self`
- **Types holding `AsyncDropGuard` members** should themselves implement `AsyncDrop` to call `async_drop()` on their members
- The `with_async_drop_2!` macro can simplify code by automatically calling `async_drop()` on scope exit, but doesn't always work
- When the macro doesn't work, call `async_drop()` manually - be very careful to call it on **each possible scope exit** (including early returns and errors)
- See `crates/utils/src/async_drop/` for implementation

## Type-Driven Invariants

- Use newtypes to enforce constraints (e.g., `BlockId` wraps fixed-size array)
- Use `NonZero` types for IDs that can't be zero
- Use enums to encode valid states (e.g., `MaybeClientId::ClientId | BlockWasDeleted`)
- Prefer compile-time guarantees over runtime checks

## Feature Flags

- Use `#[cfg(any(test, feature = "testutils"))]` for test-only code
- Keep test utilities in optional `testutils` feature
- Re-export test utilities conditionally in `lib.rs`

## Module Organization

- Keep internal modules private
- Re-export public types at crate root
- Use conditional compilation for test utilities
- Don't expose internal module structure

## Testing

- Unit tests: `#[cfg(test)]` modules next to the code being tested
- Integration tests: `crates/{crate}/tests/`
- Benchmarks: `crates/{crate}/benches/` (criterion)
- Frameworks: `#[tokio::test]`, rstest, mockall, assert_cmd
- Macro-generated test suites for testing multiple implementations

## Build Commands

```bash
cargo build --release          # Build release binary
cargo test                     # Run all tests
cargo test -p cryfs-cli        # Test specific crate
cargo fmt                      # Format code
cargo doc                      # Generate docs
```

## Key File Locations

- Entry point: `crates/cryfs-cli/src/bin/cryfs.rs`
- CLI args: `crates/cryfs-cli/src/args/`
- Core filesystem: `crates/cryfs-filesystem/src/`
- Block encryption: `crates/blockstore/src/low_level/implementations/encrypted/`
- Integrity: `crates/blockstore/src/low_level/implementations/integrity/`
- On-disk storage: `crates/blockstore/src/low_level/implementations/ondisk/`
- Ciphers: `crates/crypto/src/symmetric/`
- FUSE backend: `crates/rustfs/src/`
- Configuration: `crates/cryfs-config/src/`
- CI: `.github/workflows/ci.yml`

## Common Development Tasks

- **Adding a new cipher**: Implement in `crates/crypto/src/symmetric/`, register in config ciphers
- **Adding CLI option**: Modify `crates/cryfs-cli/src/args/`, update runner
- **Adding filesystem operation**: Implement in `crates/cryfs-filesystem/`, wire through rustfs Device trait
- **Adding tests**: Follow existing patterns, use macro fixtures for multiple implementations
