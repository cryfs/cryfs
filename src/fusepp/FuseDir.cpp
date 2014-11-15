#include <sys/types.h>
#include <sys/stat.h>
#include <fcntl.h>
#include <dirent.h>
#include <fusepp/FuseDevice.h>
#include <fusepp/FuseDir.h>
#include <fusepp/FuseErrnoException.h>
#include <fusepp/FuseFile.h>


using std::string;
using std::unique_ptr;
using std::make_unique;
using std::vector;

namespace fusepp {

FuseDir::FuseDir(FuseDevice *device, const bf::path &path)
  :FuseNode(device, path) {
  assert(bf::is_directory(base_path()));
}

FuseDir::~FuseDir() {
}

unique_ptr<FuseFile> FuseDir::createFile(const string &name, mode_t mode) {
  auto file_path = base_path() / name;
  //Create file
  int fd = ::creat(file_path.c_str(), mode);
  CHECK_RETVAL(fd);
  ::close(fd);
  return make_unique<FuseFile>(device(), path() / name);
}

unique_ptr<FuseDir> FuseDir::createDir(const string &name, mode_t mode) {
  auto dir_path = base_path() / name;
  //Create dir
  int retval = ::mkdir(dir_path.c_str(), mode);
  CHECK_RETVAL(retval);
  return make_unique<FuseDir>(device(), path() / name);
}

void FuseDir::rmdir() {
  int retval = ::rmdir(base_path().c_str());
  CHECK_RETVAL(retval);
}

unique_ptr<vector<string>> FuseDir::children() const {
  DIR *dir = ::opendir(base_path().c_str());
  if (dir == nullptr) {
    throw FuseErrnoException(errno);
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
    throw FuseErrnoException(readdir_errno);
  }
  int retval = ::closedir(dir);
  CHECK_RETVAL(retval);
  return result;
}

} /* namespace fusepp */
