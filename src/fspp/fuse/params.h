#pragma once
#ifndef MESSMER_FSPP_FUSE_PARAMS_H_
#define MESSMER_FSPP_FUSE_PARAMS_H_

#if defined(_MSC_VER)
// On Windows we don't run on libFUSE but on Dokany's FUSE wrapper, which implements FUSE 2.7 and
// has no libFUSE 3 counterpart (see https://github.com/dokan-dev/dokany/issues/182, open since
// 2016). So the Windows build uses the FUSE 2 API. Everything that differs between the two is
// guarded with FUSE_MAJOR_VERSION in Fuse.h and Fuse.cpp.
#define FUSE_USE_VERSION 27
#else
#define FUSE_USE_VERSION 39

#if defined(__APPLE__)
// macFUSE ships libfuse 3 (since macFUSE 4.10.0), but by default it replaces six fuse_operations
// members with macOS specific variants: getattr and readdir take a 'struct fuse_darwin_attr'
// instead of a 'struct stat', utimens takes a timespec[3] instead of a timespec[2], statfs takes a
// 'struct statfs' instead of a 'struct statvfs', and get/setxattr take an extra position argument.
// We implement the vanilla FUSE 3 signatures, so turn the extensions off. libfuse itself builds
// with the same define. This has to happen before <fuse.h> is included.
#define FUSE_DARWIN_ENABLE_EXTENSIONS 0
#endif
#endif

#include <fuse.h>

#if !defined(_MSC_VER) && FUSE_MAJOR_VERSION < 3
// Without this, using a libFUSE 2 header produces a wall of signature mismatches instead of saying
// what is actually wrong. Windows is exempt because Dokany's FUSE wrapper is still at FUSE 2.7.
#error "CryFS needs libFUSE 3. On macOS, install macFUSE 4.10.0 or newer - that is the first release that ships libFUSE 3."
#endif

#if FUSE_MAJOR_VERSION < 3
// Two types our own interface uses that only exist in FUSE 3. Declaring them here lets fspp keep
// one set of signatures for both APIs; only the wrappers in Fuse.cpp differ. Nothing outside those
// wrappers ever looks inside either type: the config pointer is unused, and the readdir flags are
// always FUSE_READDIR_DEFAULTS on FUSE 2, which has no readdirplus.
enum fuse_readdir_flags { FUSE_READDIR_DEFAULTS = 0 };
struct fuse_config;
#endif

#endif
