# CryFS C++ Architecture Guide

This document helps AI coding agents understand the CryFS C++ codebase structure, patterns, and conventions.

## Project Overview

CryFS is an encrypted filesystem for cloud storage (Dropbox, iCloud, OneDrive). It encrypts files while hiding file sizes, directory structure, and metadata.

- **Language:** C++17
- **Build System:** CMake with Conan package manager
- **Compilers:** GCC >= 7.0, Clang >= 7.0
- **Platforms:** Linux, macOS, Windows (experimental)

## Architecture

The architecture is layered. Dependencies flow downward (higher layers depend on lower layers):

```
┌─────────────────────────────────────────────────────────────┐
│  APPLICATION LAYER                                          │
│  cryfs-cli (main CLI binary, mounting, password prompts)    │
│  stats (cryfs-stats binary, filesystem statistics)          │
│  cryfs-unmount (unmount utility)                            │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│  CORE FILESYSTEM LAYER                                      │
│  cryfs (config, filesystem logic: CryDevice, CryFile, etc.) │
│       │                                                     │
│       └──► fspp (FUSE/Dokan abstraction, Device trait)      │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│  BLOB STORAGE LAYER                                         │
│  fsblobstore (filesystem semantics: file/dir/symlink blobs) │
│  parallelaccessfsblobstore (concurrency decorators)         │
│  cachingfsblobstore (caching decorators)                    │
│       │                                                     │
│       ▼                                                     │
│  blobstore (variable-size data as trees of blocks)          │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│  BLOCK STORAGE LAYER (decorator stack, innermost first)     │
│  OnDiskBlockStore (disk I/O)                                │
│       ↓                                                     │
│  EncryptedBlockStore (encryption via Crypto++)              │
│       ↓                                                     │
│  IntegrityBlockStore (integrity verification, versioning)   │
│       ↓                                                     │
│  CachingBlockStore (LRU caching with periodic flush)        │
│       ↓                                                     │
│  CompressingBlockStore (optional gzip/RLE compression)      │
│       ↓                                                     │
│  ParallelAccessBlockStore (thread-safe parallel access)     │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│  UTILITIES LAYER                                            │
│  cpp-utils (crypto, threading, I/O, data structures)        │
│  parallelaccessstore (generic parallel access patterns)     │
│  gitversion (version management)                            │
└─────────────────────────────────────────────────────────────┘
```

## Directory Structure

### Source Code (`src/`)

| Directory | Purpose | Key Types |
|-----------|---------|-----------|
| `cpp-utils/` | Utility library (crypto, threading, I/O, data) | `Data`, `unique_ref`, `EncryptionKey`, `Random` |
| `blockstore/` | Fixed-size encrypted blocks with decorator pattern | `BlockStore`, `BlockStore2`, `Block`, `BlockId` |
| `blobstore/` | Variable-size data on block trees | `Blob`, `BlobStore`, `BlobOnBlocks`, `DataTree` |
| `fspp/` | FUSE/Dokan filesystem abstraction | `Device`, `File`, `Dir`, `Symlink`, `OpenFile` |
| `cryfs/` | Core filesystem implementation | `CryDevice`, `CryFile`, `CryDir`, `CryConfig` |
| `cryfs-cli/` | Command-line interface and mounting | `Cli`, argument parsing |
| `cryfs-unmount/` | Unmount utility | unmount logic |
| `stats/` | Filesystem statistics tool | `cryfs-stats` binary |
| `parallelaccessstore/` | Generic parallel access decorators | `ParallelAccessStore` |
| `gitversion/` | Version tracking from git tags | version comparison |

### `cpp-utils/` Subdirectories

| Directory | Purpose |
|-----------|---------|
| `assert/` | Assertions with backtrace support |
| `crypto/` | Symmetric ciphers (AES-GCM, AES-CFB), KDF (scrypt), hashing |
| `data/` | Binary data handling, serialization |
| `io/` | Console I/O, progress bars |
| `lock/` | Lock pools, condition barriers |
| `logging/` | Logging via spdlog |
| `network/` | HTTP client for update checks (via libcurl) |
| `pointer/` | `unique_ref<T>` (non-null unique_ptr), smart pointer utilities |
| `process/` | Daemonization, subprocess handling, signal catching |
| `random/` | Random number generation |
| `system/` | Disk space, memory, time, paths |
| `tempfile/` | Temporary files and directories |
| `thread/` | Threading utilities, loop threads |

### Test Directory (`test/`)

Tests mirror the `src/` structure:

| Directory | Purpose |
|-----------|---------|
| `my-gtest-main/` | Custom GTest main with initialization |
| `cpp-utils/` | Tests for utility components |
| `blockstore/` | Tests for all blockstore implementations |
| `blobstore/` | Tests for blobstore |
| `cryfs/` | Tests for core filesystem |
| `cryfs-cli/` | Tests for CLI |
| `fspp/` | Tests for FUSE abstraction |
| `gitversion/` | Tests for version handling |
| `parallelaccessstore/` | Tests for parallel access |

### Vendor Code (`vendor/`)

| Directory | Purpose |
|-----------|---------|
| `cryptopp/` | Crypto++ library v8.9.0 for cryptographic operations |

## Key Abstractions

### `unique_ref<T>` - Non-Null Unique Pointer

A custom smart pointer that guarantees non-null:

```cpp
// src/cpp-utils/pointer/unique_ref.h
template<class T>
class unique_ref final {
    // Guarantees _target is never nullptr (checked by ASSERT)
};

// Usage:
cpputils::unique_ref<BlockStore> store = cpputils::make_unique_ref<InMemoryBlockStore>();
```

### `BlockStore` / `BlockStore2` - Block Storage Interface

```cpp
// src/blockstore/interface/BlockStore.h
class BlockStore {
    virtual BlockId createBlockId() = 0;
    virtual boost::optional<unique_ref<Block>> tryCreate(const BlockId&, Data) = 0;
    virtual boost::optional<unique_ref<Block>> load(const BlockId&) = 0;
    virtual unique_ref<Block> overwrite(const BlockId&, Data) = 0;
    virtual void remove(const BlockId&) = 0;
    virtual uint64_t numBlocks() const = 0;
    // ...
};
```

### `Blob` - Variable-Size Data Interface

```cpp
// src/blobstore/interface/Blob.h
class Blob {
    virtual const BlockId& blockId() const = 0;
    virtual uint64_t size() const = 0;
    virtual void resize(uint64_t numBytes) = 0;
    virtual void read(void* target, uint64_t offset, uint64_t size) const = 0;
    virtual void write(const void* source, uint64_t offset, uint64_t size) = 0;
    virtual void flush() = 0;
    // ...
};
```

### `Device` - FUSE Filesystem Interface

```cpp
// src/fspp/fs_interface/Device.h
class Device {
    virtual statvfs statfs() = 0;
    virtual boost::optional<unique_ref<Node>> Load(const path& path) = 0;
    virtual boost::optional<unique_ref<File>> LoadFile(const path& path) = 0;
    virtual boost::optional<unique_ref<Dir>> LoadDir(const path& path) = 0;
    virtual boost::optional<unique_ref<Symlink>> LoadSymlink(const path& path) = 0;
    // ...
};
```

## Coding Conventions

### Header Guards and Namespaces

```cpp
#pragma once
#ifndef MESSMER_COMPONENT_SUBCOMPONENT_H_
#define MESSMER_COMPONENT_SUBCOMPONENT_H_

namespace component {
// code
}

#endif
```

### Error Handling

- **Assertions:** Use `ASSERT(expr, msg)` macro for invariant checking
  - Debug builds: abort on failure
  - Release builds: throw `AssertFailed` exception
- **Exceptions:** Custom `CryfsException` with `ErrorCode` for user-facing errors
- **Optional values:** Use `boost::optional<T>` for nullable returns

```cpp
// Assertion example
ASSERT(_target.get() != nullptr, "Member was moved out");

// Exception example
throw CryfsException("Config file not found", ErrorCode::InvalidFilesystem);
```

### Error Codes

```cpp
// src/cryfs/impl/ErrorCodes.h
enum class ErrorCode : int {
    Success = 0,
    UnspecifiedError = 1,
    InvalidArguments = 10,
    WrongPassword = 11,
    // ... more specific codes
};
```

### Smart Pointers

- Use `cpputils::unique_ref<T>` for non-nullable ownership
- Use `boost::optional<cpputils::unique_ref<T>>` for nullable ownership
- Use `std::shared_ptr<T>` for shared ownership

### Macros

```cpp
// src/cpp-utils/macros.h
#define DISALLOW_COPY_AND_ASSIGN(TypeName) \
    TypeName(const TypeName&) = delete; \
    void operator=(const TypeName&) = delete
```

## Build System

### Dependencies (via Conan)

| Library | Version | Purpose |
|---------|---------|---------|
| Boost | 1.84.0 | Filesystem, threading, program options |
| spdlog | 1.14.1 | Logging |
| range-v3 | cci.20240905 | C++ ranges |
| libcurl | 8.9.1 | HTTP for update checks (optional) |
| GTest | 1.15.0 | Unit testing (if tests enabled) |

### Platform Dependencies

| Platform | Dependency | Purpose |
|----------|-----------|---------|
| Linux/macOS | libFUSE >= 2.9 | Filesystem in Userspace |
| macOS | macFUSE | FUSE for macOS |
| Windows | Dokan 2.2.0 | Windows filesystem driver |

### Build Commands

```bash
# Install conan (first time)
pipx install conan~=2.7.0
conan profile detect

# Build release
conan build . -s build_type=RelWithDebInfo --build=missing

# Build with tests
conan build . -s build_type=Debug --build=missing -o "&:build_tests=True"

# Run tests (after building with tests)
cd build/Debug/test
./cpp-utils/cpp-utils-test
./blockstore/blockstore-test
./blobstore/blobstore-test
./cryfs/cryfs-test
./cryfs-cli/cryfs-cli-test
./fspp/fspp-test
./gitversion/gitversion-test
./parallelaccessstore/parallelaccessstore-test

# Build without conan (local dependencies)
mkdir build && cd build
cmake .. -DCMAKE_BUILD_TYPE=RelWithDebInfo -DBUILD_TESTING=ON
make

# Create packages
cd build/RelWithDebInfo && make package
```

### CMake Options

| Option | Default | Description |
|--------|---------|-------------|
| `BUILD_TESTING` | OFF | Build test cases |
| `CRYFS_UPDATE_CHECKS` | ON | Enable online update/security checks |
| `DISABLE_OPENMP` | OFF | Build without OpenMP (slower) |
| `USE_WERROR` | OFF | Treat warnings as errors |
| `USE_CLANG_TIDY` | OFF | Enable clang-tidy checks |

### Conan Options

| Option | Default | Description |
|--------|---------|-------------|
| `build_tests` | False | Build test cases |
| `update_checks` | True | Enable update checks |
| `disable_openmp` | False | Disable OpenMP |
| `use_werror` | False | Treat warnings as errors |

## Testing

### Framework

- **Google Test (GTest)** for unit testing
- **Google Mock (GMock)** for mocking
- Custom `my-gtest-main` for test initialization

### Test Patterns

**Generic vs Specific Tests:**
- `*Test_Generic.cpp` - Tests that apply to multiple implementations (parameterized)
- `*Test_Specific.cpp` - Implementation-specific tests

**Test Fixtures:**
```cpp
// Generic test using typed test suite
template<class Cipher>
class EncryptedBlockStoreTestFixture: public BlockStoreTestFixture {
public:
    unique_ref<BlockStore> createBlockStore() override {
        return make_unique_ref<LowToHighLevelBlockStore>(
            make_unique_ref<EncryptedBlockStore2<Cipher>>(
                make_unique_ref<InMemoryBlockStore2>(), createKeyFixture()
            )
        );
    }
};

INSTANTIATE_TYPED_TEST_SUITE_P(Encrypted_AES256_GCM, BlockStoreTest,
                                EncryptedBlockStoreTestFixture<AES256_GCM>);
```

### Test Utilities

- `FakeBlockStore` - In-memory fake for testing
- `MockBlockStore` - GMock-based mock
- `DataFixture` - Generate deterministic test data

## Key File Locations

| Purpose | Location |
|---------|----------|
| Entry point | `src/cryfs-cli/main.cpp` |
| CLI logic | `src/cryfs-cli/Cli.cpp` |
| Core filesystem | `src/cryfs/impl/filesystem/CryDevice.cpp` |
| Block encryption | `src/blockstore/implementations/encrypted/` |
| Integrity checking | `src/blockstore/implementations/integrity/` |
| On-disk storage | `src/blockstore/implementations/ondisk/` |
| Ciphers | `src/cpp-utils/crypto/symmetric/` |
| FUSE backend | `src/fspp/fuse/` |
| Configuration | `src/cryfs/impl/config/` |
| CI | `.github/workflows/main.yaml` |

## Common Development Tasks

### Adding a New Cipher

1. Implement cipher class in `src/cpp-utils/crypto/symmetric/`
2. Register in `src/cpp-utils/crypto/symmetric/ciphers.h`
3. Add to config cipher list in `src/cryfs/impl/config/`
4. Add tests in `test/cpp-utils/crypto/symmetric/`

### Adding a CLI Option

1. Modify argument parsing in `src/cryfs-cli/`
2. Update `Cli.cpp` to handle new option
3. Add tests in `test/cryfs-cli/`

### Adding a BlockStore Implementation

1. Create implementation in `src/blockstore/implementations/<name>/`
2. Implement `BlockStore` or `BlockStore2` interface
3. Add CMakeLists.txt entry
4. Add generic tests via fixture pattern
5. Add specific tests if needed

### Adding a Filesystem Operation

1. Implement in `src/cryfs/impl/filesystem/` (CryDevice, CryFile, etc.)
2. Wire through fspp Device trait if needed
3. Add tests in `test/cryfs/` and `test/fspp/`

## Static Analysis

### Clang-Tidy

Configuration in `.clang-tidy`:
- Enabled checks: `clang-analyzer-*`, `bugprone-*`, `cert-*`, `cppcoreguidelines-*`, `misc-*`
- Run: `./run-clang-tidy.sh`

### Include-What-You-Use (IWYU)

- Run: `./run-iwyu.sh`

## Platform Notes

### Windows (Experimental)

- Uses Dokan instead of FUSE
- Requires Visual Studio 2019/2022
- Some tests are disabled on Windows (see CI config)

### macOS

- Requires macFUSE from https://osxfuse.github.io/
- Apple Clang support varies by macOS version

## Stability Notes

- CryFS does not support concurrent access from multiple devices
- No journaling - power loss during write can corrupt filesystem
- Disk full during write can corrupt filesystem
- No fsck-like recovery tool available yet (in development)
