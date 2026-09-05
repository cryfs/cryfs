#pragma once
#ifndef MESSMER_FSPP_FUSE_PARAMS_H_
#define MESSMER_FSPP_FUSE_PARAMS_H_

// 309, not 39. libfuse changed FUSE_MAKE_VERSION from (major*10 + minor) to (major*100 + minor) in
// 3.10.0, so on every header released since then 39 is not 3.9, it is a number that means nothing
// and happens to land on the right side of each of libfuse's remaining version guards. 309 is
// FUSE_MAKE_VERSION(3, 9) on a modern header and behaves exactly like 39 on an older one. Spelling
// it as FUSE_MAKE_VERSION(3, 9) would be nicer still, but this header is shared with the Windows
// build and Dokany's FUSE 2.7 headers don't define that macro.
#define FUSE_USE_VERSION 309

#if defined(__APPLE__)
// macFUSE ships libfuse 3 (since macFUSE 4.10.0), but by default it replaces six fuse_operations
// members with macOS specific variants: getattr and readdir take a 'struct fuse_darwin_attr'
// instead of a 'struct stat', utimens takes a timespec[3] instead of a timespec[2], statfs takes a
// 'struct statfs' instead of a 'struct statvfs', and get/setxattr take an extra position argument.
// We implement the vanilla FUSE 3 signatures, so turn the extensions off. libfuse itself builds
// with the same define. This has to happen before <fuse.h> is included.
#define FUSE_DARWIN_ENABLE_EXTENSIONS 0
#endif

#include <fuse.h>

#if !defined(_MSC_VER) && FUSE_MAJOR_VERSION < 3
// Without this, using a libFUSE 2 header produces a wall of signature mismatches instead of saying
// what is actually wrong. Windows is exempt because Dokany's FUSE wrapper is still at FUSE 2.7.
#error "CryFS needs libFUSE 3. On macOS, install macFUSE 4.10.0 or newer - that is the first release that ships libFUSE 3."
#endif

#endif
