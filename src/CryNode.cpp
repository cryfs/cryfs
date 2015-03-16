#include "CryNode.h"

#include <sys/time.h>

#include "CryDevice.h"
#include "CryDir.h"
#include "CryFile.h"
#include "messmer/fspp/fuse/FuseErrnoException.h"

namespace bf = boost::filesystem;

//TODO Get rid of this in favor of an exception hierarchy
using fspp::fuse::CHECK_RETVAL;
using fspp::fuse::FuseErrnoException;

namespace cryfs {

CryNode::CryNode() {
}

CryNode::~CryNode() {
}

void CryNode::access(int mask) const {
  return;
  throw FuseErrnoException(ENOTSUP);
}

void CryNode::rename(const bf::path &to) {
  throw FuseErrnoException(ENOTSUP);
}

void CryNode::utimens(const timespec times[2]) {
  throw FuseErrnoException(ENOTSUP);
}

} /* namespace cryfs */
