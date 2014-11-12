#include "../cryfs_lib/CryDevice.h"

#include <memory>
#include <fcntl.h>

#include "CryDir.h"
#include "CryFile.h"
#include "CryOpenFile.h"
#include "CryErrnoException.h"
#include "utils/pointer.h"

#include "CryOpenDir.h"

using namespace cryfs;

using std::unique_ptr;
using std::make_unique;
using std::vector;
using std::string;

CryDevice::CryDevice(const bf::path &rootdir)
  :_rootdir(rootdir), _open_files(), _open_dirs() {
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

unique_ptr<CryFile> CryDevice::LoadFile(const bf::path &path) {
  auto node = Load(path);
  auto file = dynamic_pointer_move<CryFile>(node);
  if (!file) {
	  throw CryErrnoException(EISDIR);
  }
  return file;
}

unique_ptr<CryDir> CryDevice::LoadDir(const bf::path &path) {
  auto node = Load(path);
  auto dir = dynamic_pointer_move<CryDir>(node);
  if (!dir) {
    throw CryErrnoException(ENOTDIR);
  }
  return dir;
}

int CryDevice::openFile(const bf::path &path, int flags) {
  auto file = LoadFile(path);
  return openFile(*file, flags);
}

int CryDevice::openFile(const CryFile &file, int flags) {
  return _open_files.open(file, flags);
}

void CryDevice::closeFile(int descriptor) {
  _open_files.close(descriptor);
}

void CryDevice::lstat(const bf::path &path, struct ::stat *stbuf) {
  Load(path)->stat(stbuf);
}

void CryDevice::fstat(int descriptor, struct ::stat *stbuf) {
  _open_files.get(descriptor)->stat(stbuf);
}

void CryDevice::truncate(const bf::path &path, off_t size) {
  LoadFile(path)->truncate(size);
}

void CryDevice::ftruncate(int descriptor, off_t size) {
  _open_files.get(descriptor)->truncate(size);
}

int CryDevice::read(int descriptor, void *buf, size_t count, off_t offset) {
  return _open_files.get(descriptor)->read(buf, count, offset);
}

void CryDevice::write(int descriptor, const void *buf, size_t count, off_t offset) {
  _open_files.get(descriptor)->write(buf, count, offset);
}

void CryDevice::fsync(int descriptor) {
  _open_files.get(descriptor)->fsync();
}

void CryDevice::fdatasync(int descriptor) {
  _open_files.get(descriptor)->fdatasync();
}

void CryDevice::access(const bf::path &path, int mask) {
  Load(path)->access(mask);
}

int CryDevice::createAndOpenFile(const bf::path &path, mode_t mode) {
  //TODO Creating the file opens and closes it. We then reopen it afterwards.
  //     This is slow. Improve!
  auto dir = LoadDir(path.parent_path());
  auto file = dir->createFile(path.filename().native(), mode);
  return openFile(*file, O_WRONLY | O_TRUNC);
}

void CryDevice::mkdir(const bf::path &path, mode_t mode) {
  auto dir = LoadDir(path.parent_path());
  dir->createDir(path.filename().native(), mode);
}

void CryDevice::rmdir(const bf::path &path) {
  auto dir = LoadDir(path);
  dir->rmdir();
}

void CryDevice::unlink(const bf::path &path) {
  auto file = LoadFile(path);
  file->unlink();
}

void CryDevice::rename(const bf::path &from, const bf::path &to) {
  auto node = Load(from);
  node->rename(to);
}

int CryDevice::openDir(const bf::path &path) {
  auto dir = LoadDir(path);
  return _open_dirs.open(*dir);
}

unique_ptr<vector<string>> CryDevice::readDir(int descriptor) {
  return _open_dirs.get(descriptor)->readdir();
}

void CryDevice::closeDir(int descriptor) {
  _open_dirs.close(descriptor);
}

void CryDevice::utimens(const bf::path &path, const timespec times[2]) {
  auto node = Load(path);
  node->utimens(times);
}

void CryDevice::statfs(const bf::path &path, struct statvfs *fsstat) {
  int retval = ::statvfs(path.c_str(), fsstat);
  CHECK_RETVAL(retval);
}
