#include "CryDir.h"

#include <sys/types.h>
#include <sys/stat.h>
#include <fcntl.h>

#include "CryDevice.h"
#include "CryFile.h"
#include "CryErrnoException.h"

using std::string;
using std::unique_ptr;
using std::make_unique;

namespace cryfs {

CryDir::CryDir(CryDevice *device, const bf::path &path)
  :CryNode(device, path) {
  assert(bf::is_directory(base_path()));
}

CryDir::~CryDir() {
}

unique_ptr<CryFile> CryDir::createFile(const string &name, mode_t mode) {
  auto file_path = base_path() / name;
  //Create file
  int fd = ::creat(file_path.c_str(), mode);
  CHECK_RETVAL(fd);
  ::close(fd);
  return make_unique<CryFile>(device(), path() / name);
}

unique_ptr<CryDir> CryDir::createDir(const string &name, mode_t mode) {
  auto dir_path = base_path() / name;
  //Create dir
  int retval = ::mkdir(dir_path.c_str(), mode);
  CHECK_RETVAL(retval);
  return make_unique<CryDir>(device(), path() / name);
}

void CryDir::rmdir() {
  int retval = ::rmdir(base_path().c_str());
  CHECK_RETVAL(retval);
}

} /* namespace cryfs */
