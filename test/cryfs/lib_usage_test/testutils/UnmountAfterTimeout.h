#pragma once
#ifndef CRYFS_TEST_LIBUSAGETEST_TESTUTILS_UNMOUNTAFTERTIMEOUT_H
#define CRYFS_TEST_LIBUSAGETEST_TESTUTILS_UNMOUNTAFTERTIMEOUT_H

#include <boost/filesystem/path.hpp>
#include <boost/thread.hpp>
#include <cryfs/cryfs.h>
#include <cpp-utils/macros.h>

class UnmountAfterTimeout final {
public:
    UnmountAfterTimeout(const boost::filesystem::path &mountdir): _unmountThread(), _timeoutPassed(false) {
      _unmountThread = boost::thread([mountdir, this]() {
          boost::this_thread::sleep_for(TIMEOUT);
          _timeoutPassed = true;
          if (cryfs_success != cryfs_unmount(mountdir.native().c_str(), mountdir.native().size())) {
              std::cerr << "Unmounting failed" << std::endl;
              exit(1); // Exit full process, because the EXPECT_ macros of gtest don't work in a non-main thread.
          }
      });
    }

    ~UnmountAfterTimeout() {
        _unmountThread.join();
    }

    bool timeoutPassed() {
        return _timeoutPassed;
    }

private:
    static constexpr boost::chrono::seconds TIMEOUT = boost::chrono::seconds(2);
    boost::thread _unmountThread;
    bool _timeoutPassed;

    DISALLOW_COPY_AND_ASSIGN(UnmountAfterTimeout);
};

#endif
