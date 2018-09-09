#pragma once
#ifndef MESSMER_FSPP_TEST_TESTUTILS_OPENFILEHANDLE_H_
#define MESSMER_FSPP_TEST_TESTUTILS_OPENFILEHANDLE_H_

#include <fcntl.h>
#include <cpp-utils/macros.h>
#include <errno.h>
#include <thread>
#include <chrono>
#if defined(_MSC_VER)
#include <io.h>
#else
#include <unistd.h>
#endif

class OpenFileHandle final {
public:
    OpenFileHandle(const char *path, int flags): fd_(::open(path, flags)), errno_(fd_ >= 0 ? 0 : errno) {
    }

    OpenFileHandle(const char *path, int flags, int flags2): fd_(::open(path, flags, flags2)), errno_(fd_ >= 0 ? 0 : errno) {
    }

    ~OpenFileHandle() {
        if (fd_ >= 0) {
            ::close(fd_);
#ifdef __APPLE__
            // On Mac OS X, we might have to give it some time to free up the file
            std::this_thread::sleep_for(std::chrono::milliseconds(50));
#endif
        }
    }

    int fd() { return fd_; }
    int errorcode() { return errno_; }

    void release() {
        fd_ = -1; // don't close anymore
    }

private:
    int fd_;
    const int errno_;

    DISALLOW_COPY_AND_ASSIGN(OpenFileHandle);
};


#endif
