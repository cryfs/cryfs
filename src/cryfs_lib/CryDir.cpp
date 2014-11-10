#include "CryDir.h"

#include <sys/types.h>
#include <sys/stat.h>
#include <fcntl.h>

#include "CryDevice.h"
#include "CryErrnoException.h"

using std::string;
using std::unique_ptr;

namespace cryfs {

CryDir::CryDir(CryDevice *device, const bf::path &path)
  :CryNode(device, path) {
}

CryDir::~CryDir() {
}

void CryDir::createFile(const string &name, mode_t mode) {
  auto file_path = base_path() / name;
  //Create file
  int fd = ::creat(file_path.c_str(), mode);
  CHECK_RETVAL(fd);
  ::close(fd);
}

} /* namespace cryfs */
