#include "FuseDevice.h"

#include <memory>
#include <fcntl.h>
#include <fusepp/FuseDir.h>
#include <fusepp/FuseErrnoException.h>
#include <fusepp/FuseFile.h>
#include <fusepp/FuseOpenFile.h>

#include "utils/pointer.h"

using namespace fusepp;

using std::unique_ptr;
using std::make_unique;
using std::vector;
using std::string;

FuseDevice::FuseDevice(const bf::path &rootdir)
  :_rootdir(rootdir), _open_files() {
}

FuseDevice::~FuseDevice() {
}

unique_ptr<FuseNode> FuseDevice::Load(const bf::path &path) {
  auto real_path = RootDir() / path;
  if(bf::is_directory(real_path)) {
    return make_unique<FuseDir>(this, path);
  } else if(bf::is_regular_file(real_path)) {
    return make_unique<FuseFile>(this, path);
  }

  throw FuseErrnoException(ENOENT);
}

unique_ptr<FuseFile> FuseDevice::LoadFile(const bf::path &path) {
  auto node = Load(path);
  auto file = dynamic_pointer_move<FuseFile>(node);
  if (!file) {
	  throw FuseErrnoException(EISDIR);
  }
  return file;
}

unique_ptr<FuseDir> FuseDevice::LoadDir(const bf::path &path) {
  auto node = Load(path);
  auto dir = dynamic_pointer_move<FuseDir>(node);
  if (!dir) {
    throw FuseErrnoException(ENOTDIR);
  }
  return dir;
}

int FuseDevice::openFile(const bf::path &path, int flags) {
  auto file = LoadFile(path);
  return openFile(*file, flags);
}

int FuseDevice::openFile(const FuseFile &file, int flags) {
  return _open_files.open(file, flags);
}

void FuseDevice::closeFile(int descriptor) {
  _open_files.close(descriptor);
}

void FuseDevice::lstat(const bf::path &path, struct ::stat *stbuf) {
  Load(path)->stat(stbuf);
}

void FuseDevice::fstat(int descriptor, struct ::stat *stbuf) {
  _open_files.get(descriptor)->stat(stbuf);
}

void FuseDevice::truncate(const bf::path &path, off_t size) {
  LoadFile(path)->truncate(size);
}

void FuseDevice::ftruncate(int descriptor, off_t size) {
  _open_files.get(descriptor)->truncate(size);
}

int FuseDevice::read(int descriptor, void *buf, size_t count, off_t offset) {
  return _open_files.get(descriptor)->read(buf, count, offset);
}

void FuseDevice::write(int descriptor, const void *buf, size_t count, off_t offset) {
  _open_files.get(descriptor)->write(buf, count, offset);
}

void FuseDevice::fsync(int descriptor) {
  _open_files.get(descriptor)->fsync();
}

void FuseDevice::fdatasync(int descriptor) {
  _open_files.get(descriptor)->fdatasync();
}

void FuseDevice::access(const bf::path &path, int mask) {
  Load(path)->access(mask);
}

int FuseDevice::createAndOpenFile(const bf::path &path, mode_t mode) {
  //TODO Creating the file opens and closes it. We then reopen it afterwards.
  //     This is slow. Improve!
  auto dir = LoadDir(path.parent_path());
  auto file = dir->createFile(path.filename().native(), mode);
  return openFile(*file, O_WRONLY | O_TRUNC);
}

void FuseDevice::mkdir(const bf::path &path, mode_t mode) {
  auto dir = LoadDir(path.parent_path());
  dir->createDir(path.filename().native(), mode);
}

void FuseDevice::rmdir(const bf::path &path) {
  auto dir = LoadDir(path);
  dir->rmdir();
}

void FuseDevice::unlink(const bf::path &path) {
  auto file = LoadFile(path);
  file->unlink();
}

void FuseDevice::rename(const bf::path &from, const bf::path &to) {
  auto node = Load(from);
  node->rename(to);
}

unique_ptr<vector<string>> FuseDevice::readDir(const bf::path &path) {
  auto dir = LoadDir(path);
  return dir->children();
}

void FuseDevice::utimens(const bf::path &path, const timespec times[2]) {
  auto node = Load(path);
  node->utimens(times);
}

void FuseDevice::statfs(const bf::path &path, struct statvfs *fsstat) {
  int retval = ::statvfs(path.c_str(), fsstat);
  CHECK_RETVAL(retval);
}
