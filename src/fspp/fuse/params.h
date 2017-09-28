#pragma once
#ifndef MESSMER_FSPP_FUSE_PARAMS_H_
#define MESSMER_FSPP_FUSE_PARAMS_H_

#define FUSE_USE_VERSION 26
#if defined(__linux__) || defined(__FreeBSD__)
#include <fuse.h>
#elif __APPLE__
#include <osxfuse/fuse.h>
#else
#error System not supported
#endif

#endif
