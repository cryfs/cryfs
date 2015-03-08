#include "CryDir.h"

#include <sys/types.h>
#include <sys/stat.h>
#include <fcntl.h>
#include <dirent.h>

#include "messmer/fspp/fuse/FuseErrnoException.h"
#include "CryDevice.h"
#include "CryFile.h"

//TODO Get rid of this in favor of exception hierarchy
using fspp::fuse::CHECK_RETVAL;
using fspp::fuse::FuseErrnoException;

namespace bf = boost::filesystem;

using std::unique_ptr;
using std::make_unique;
using std::string;
using std::vector;

namespace cryfs {

CryDir::CryDir(CryDevice *device, unique_ptr<DirBlob> blob)
: _device(device), _blob(std::move(blob)) {
}

CryDir::~CryDir() {
}

void CryDir::stat(struct ::stat *result) const {
  result->st_mode = S_IFDIR | S_IRUSR | S_IXUSR | S_IWUSR;
  return;
  throw FuseErrnoException(ENOTSUP);
}

unique_ptr<fspp::File> CryDir::createFile(const string &name, mode_t mode) {
  auto child = _device->CreateBlob();
  _blob->AddChild(name, child->key());
  //TODO Do we need a return value in createDir for fspp? If not, change fspp!
  auto fileblob = make_unique<FileBlob>(std::move(child));
  fileblob->InitializeEmptyFile();
  return make_unique<CryFile>(_device, std::move(fileblob));
}

unique_ptr<fspp::Dir> CryDir::createDir(const string &name, mode_t mode) {
  auto child = _device->CreateBlob();
  _blob->AddChild(name, child->key());
  //TODO I don't think we need a return value in createDir for fspp. Change fspp!
  auto dirblob = make_unique<DirBlob>(std::move(child));
  dirblob->InitializeEmptyDir();
  return make_unique<CryDir>(_device, std::move(dirblob));
}

void CryDir::rmdir() {
  throw FuseErrnoException(ENOTSUP);
}

unique_ptr<vector<string>> CryDir::children() const {
  return _blob->GetChildren();
}

}
