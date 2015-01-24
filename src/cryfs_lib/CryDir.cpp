#include "CryDir.h"

#include <sys/types.h>
#include <sys/stat.h>
#include <fcntl.h>
#include <dirent.h>

#include "fspp/fuse/FuseErrnoException.h"
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

CryDir::CryDir(CryDevice *device, unique_ptr<DirBlock> block)
: _device(device), _block(std::move(block)) {
}

CryDir::~CryDir() {
}

unique_ptr<fspp::File> CryDir::createFile(const string &name, mode_t mode) {
  auto child = _device->CreateBlock(0);
  _block->AddChild(name, child->key());
  //TODO Di we need a return value in createDir for fspp? If not, change fspp!
  auto fileblock = make_unique<FileBlock>(std::move(child));
  fileblock->InitializeEmptyFile();
  return make_unique<CryFile>(std::move(fileblock));
}

unique_ptr<fspp::Dir> CryDir::createDir(const string &name, mode_t mode) {
  auto child = _device->CreateBlock(CryDevice::DIR_BLOCKSIZE);
  _block->AddChild(name, child->key());
  //TODO I don't think we need a return value in createDir for fspp. Change fspp!
  auto dirblock = make_unique<DirBlock>(std::move(child));
  dirblock->InitializeEmptyDir();
  return make_unique<CryDir>(_device, std::move(dirblock));
}

void CryDir::rmdir() {
  throw FuseErrnoException(ENOTSUP);
}

unique_ptr<vector<string>> CryDir::children() const {
  return _block->GetChildren();
}

}
