#include "CryNode.h"

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

} /* namespace cryfs */
