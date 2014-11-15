#include <fusepp/FuseDevice.h>
#include <fusepp/FuseErrnoException.h>
#include <fusepp/FuseNode.h>
#include <sys/time.h>


namespace fusepp {

FuseNode::FuseNode(FuseDevice *device, const bf::path &path)
  :_device(device), _path(path) {
}

FuseNode::~FuseNode() {
}

void FuseNode::stat(struct ::stat *result) const {
  int retval = ::lstat(base_path().c_str(), result);
  CHECK_RETVAL(retval);
}

void FuseNode::access(int mask) const {
  int retval = ::access(base_path().c_str(), mask);
  CHECK_RETVAL(retval);
}

void FuseNode::rename(const bf::path &to) {
  auto new_base_path = device()->RootDir() / to;
  int retval = ::rename(base_path().c_str(), new_base_path.c_str());
  CHECK_RETVAL(retval);
  _path = to;
}

void FuseNode::utimens(const timespec times[2]) {
  struct timeval timevals[2];
  TIMESPEC_TO_TIMEVAL(&timevals[0], &times[0]);
  TIMESPEC_TO_TIMEVAL(&timevals[1], &times[1]);
  ::lutimes(base_path().c_str(), timevals);
}

} /* namespace fusepp */
