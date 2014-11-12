#include "CryNode.h"

#include <sys/time.h>

#include "CryDevice.h"
#include "CryErrnoException.h"

namespace cryfs {

CryNode::CryNode(CryDevice *device, const bf::path &path)
  :_device(device), _path(path) {
}

CryNode::~CryNode() {
}

void CryNode::stat(struct ::stat *result) const {
  int retval = ::lstat(base_path().c_str(), result);
  CHECK_RETVAL(retval);
}

void CryNode::access(int mask) const {
  int retval = ::access(base_path().c_str(), mask);
  CHECK_RETVAL(retval);
}

void CryNode::rename(const bf::path &to) {
  auto new_base_path = device()->RootDir() / to;
  int retval = ::rename(base_path().c_str(), new_base_path.c_str());
  CHECK_RETVAL(retval);
  _path = to;
}

void CryNode::utimens(const timespec times[2]) {
  struct timeval timevals[2];
  TIMESPEC_TO_TIMEVAL(&timevals[0], &times[0]);
  TIMESPEC_TO_TIMEVAL(&timevals[1], &times[1]);
  ::lutimes(base_path().c_str(), timevals);
}

} /* namespace cryfs */
