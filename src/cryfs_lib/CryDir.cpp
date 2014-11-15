#include "CryDir.h"

#include <sys/types.h>
#include <sys/stat.h>
#include <fcntl.h>
#include <dirent.h>

#include "fusepp/impl/FuseErrnoException.h"
#include "CryDevice.h"
#include "CryFile.h"

//TODO Get rid of this in favor of exception hierarchy
using fspp::CHECK_RETVAL;

namespace bf = boost::filesystem;

using std::unique_ptr;
using std::make_unique;
using std::string;
using std::vector;

namespace cryfs {

CryDir::CryDir(CryDevice *device, const bf::path &path)
  :CryNode(device, path) {
  assert(bf::is_directory(base_path()));
}

CryDir::~CryDir() {
}

unique_ptr<fspp::File> CryDir::createFile(const string &name, mode_t mode) {
  auto file_path = base_path() / name;
  //Create file
  int fd = ::creat(file_path.c_str(), mode);
  CHECK_RETVAL(fd);
  ::close(fd);
  return make_unique<CryFile>(device(), path() / name);
}

unique_ptr<fspp::Dir> CryDir::createDir(const string &name, mode_t mode) {
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

unique_ptr<vector<string>> CryDir::children() const {
  DIR *dir = ::opendir(base_path().c_str());
  if (dir == nullptr) {
    throw fspp::FuseErrnoException(errno);
  }

  // Set errno=0 so we can detect whether it changed later
  errno = 0;

  auto result = make_unique<vector<string>>();

  struct dirent *entry = ::readdir(dir);
  while(entry != nullptr) {
    result->push_back(entry->d_name);
    entry = ::readdir(dir);
  }
  //On error, ::readdir returns nullptr and sets errno.
  if (errno != 0) {
    int readdir_errno = errno;
    ::closedir(dir);
    throw fspp::FuseErrnoException(readdir_errno);
  }
  int retval = ::closedir(dir);
  CHECK_RETVAL(retval);
  return result;
}

} /* namespace cryfs */
