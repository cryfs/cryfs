#pragma once
#ifndef MESSMER_FSPP_TEST_TESTUTILS_FUSETHREAD_H_
#define MESSMER_FSPP_TEST_TESTUTILS_FUSETHREAD_H_

#include <boost/thread.hpp>
#include <boost/chrono.hpp>
#include <messmer/cpp-utils/macros.h>

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
  boost::thread _child;

  DISALLOW_COPY_AND_ASSIGN(FuseThread);
};

#endif
