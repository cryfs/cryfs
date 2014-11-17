#pragma once
#ifndef TEST_TESTUTILS_FUSETHREAD_H_
#define TEST_TESTUTILS_FUSETHREAD_H_

#include <thread>

namespace fspp {
namespace fuse {
  class Fuse;
}
}

class FuseThread {
public:
  FuseThread(fspp::fuse::Fuse *fuse);
  void start(int argc, char *argv[]);
  void stop();

private:
  fspp::fuse::Fuse *_fuse;
  std::thread _child;
};

#endif
