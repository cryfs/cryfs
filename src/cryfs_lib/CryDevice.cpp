#include "../cryfs_lib/CryDevice.h"

#include <memory>

#include "CryDir.h"
#include "CryFile.h"
#include "CryOpenFile.h"
#include "CryErrnoException.h"
#include "utils/pointer.h"

using namespace cryfs;

using std::unique_ptr;
using std::make_unique;

CryDevice::CryDevice(const bf::path &rootdir)
  :_rootdir(rootdir), _open_files() {
}

CryDevice::~CryDevice() {
}

unique_ptr<CryNode> CryDevice::Load(const bf::path &path) {
  auto real_path = RootDir() / path;
  if(bf::is_directory(real_path)) {
    return make_unique<CryDir>(this, path);
  } else if(bf::is_regular_file(real_path)) {
    return make_unique<CryFile>(this, path);
  }

  throw CryErrnoException(ENOENT);
}

std::unique_ptr<CryFile> CryDevice::LoadFile(const bf::path &path) {
  auto node = Load(path);
  auto file = dynamic_pointer_move<CryFile>(node);
  if (!file) {
	throw CryErrnoException(EISDIR);
  }
  return file;
}

int CryDevice::OpenFile(const bf::path &path, int flags) {
  auto file = LoadFile(path);
  return _open_files.open(*file, flags);
}

void CryDevice::lstat(const bf::path &path, struct ::stat *stbuf) {
  Load(path)->stat(stbuf);
}

void CryDevice::fstat(int descriptor, struct ::stat *stbuf) {
  _open_files.get(descriptor)->stat(stbuf);
}
