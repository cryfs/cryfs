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
#endif

#include <fuse.h>

#if FUSE_MAJOR_VERSION < 3
// Two types our own interface uses that only exist in FUSE 3. Declaring them here lets fspp keep
// one set of signatures for both APIs; only the wrappers in Fuse.cpp differ. Nothing outside those
// wrappers ever looks inside either type: the config pointer is unused, and the readdir flags are
// always FUSE_READDIR_DEFAULTS on FUSE 2, which has no readdirplus.
enum fuse_readdir_flags { FUSE_READDIR_DEFAULTS = 0 };
struct fuse_config;
#endif

#endif
