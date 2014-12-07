#include "CryNode.h"

#include <sys/time.h>

#include "CryDevice.h"
#include "fspp/fuse/FuseErrnoException.h"

namespace bf = boost::filesystem;

//TODO Get rid of this in favor of an exception hierarchy
using fspp::fuse::CHECK_RETVAL;
using fspp::fuse::FuseErrnoException;

namespace cryfs {

CryNode::CryNode() {
}

CryNode::~CryNode() {
}

void CryNode::stat(struct ::stat *result) const {
  throw FuseErrnoException(ENOTSUP);
}

void CryNode::access(int mask) const {
  throw FuseErrnoException(ENOTSUP);
}

void CryNode::rename(const bf::path &to) {
  throw FuseErrnoException(ENOTSUP);
}

void CryNode::utimens(const timespec times[2]) {
  throw FuseErrnoException(ENOTSUP);
}

} /* namespace cryfs */
