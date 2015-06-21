#include "FilesystemImpl.h"

#include <fcntl.h>
#include "../fs_interface/Device.h"
#include "../fs_interface/Dir.h"
#include "../fs_interface/Symlink.h"

#include "../fuse/FuseErrnoException.h"
#include "../fs_interface/File.h"


#include <messmer/cpp-utils/pointer/unique_ref.h>

using namespace fspp;
using cpputils::dynamic_pointer_move;
using cpputils::unique_ref;
using cpputils::make_unique_ref;
using std::vector;
using std::string;
using boost::none;

namespace bf = boost::filesystem;

FilesystemImpl::FilesystemImpl(Device *device)
  :_device(device), _open_files() {
}

FilesystemImpl::~FilesystemImpl() {
}

unique_ref<File> FilesystemImpl::LoadFile(const bf::path &path) {
  auto node = _device->Load(path);
  if (node == none) {
    throw fuse::FuseErrnoException(EIO);
  }
  auto file = dynamic_pointer_move<File>(*node);
  if (file == none) {
	  throw fuse::FuseErrnoException(EISDIR);
  }
  return std::move(*file);
}

unique_ref<Dir> FilesystemImpl::LoadDir(const bf::path &path) {
  auto node = _device->Load(path);
  if (node == none) {
    throw fuse::FuseErrnoException(EIO);
  }
  auto dir = dynamic_pointer_move<Dir>(*node);
  if (dir == none) {
    throw fuse::FuseErrnoException(ENOTDIR);
  }
  return std::move(*dir);
}

unique_ref<Symlink> FilesystemImpl::LoadSymlink(const bf::path &path) {
  auto node = _device->Load(path);
  if (node == none) {
    throw fuse::FuseErrnoException(EIO);
  }
  auto lnk = dynamic_pointer_move<Symlink>(*node);
  if (lnk == none) {
    throw fuse::FuseErrnoException(ENOTDIR);
  }
  return std::move(*lnk);
}

int FilesystemImpl::openFile(const bf::path &path, int flags) {
  auto file = LoadFile(path);
  return openFile(*file, flags);
}

int FilesystemImpl::openFile(const File &file, int flags) {
  return _open_files.open(file.open(flags));
}

void FilesystemImpl::flush(int descriptor) {
  _open_files.get(descriptor)->flush();
}

void FilesystemImpl::closeFile(int descriptor) {
  _open_files.close(descriptor);
}

void FilesystemImpl::lstat(const bf::path &path, struct ::stat *stbuf) {
  auto node = _device->Load(path);
  if(node == none) {
    throw fuse::FuseErrnoException(ENOENT);
  } else {
    (*node)->stat(stbuf);
  }
}

void FilesystemImpl::fstat(int descriptor, struct ::stat *stbuf) {
  _open_files.get(descriptor)->stat(stbuf);
}

void FilesystemImpl::chmod(const boost::filesystem::path &path, mode_t mode) {
  auto node = _device->Load(path);
  if(node == none) {
    throw fuse::FuseErrnoException(ENOENT);
  } else {
    (*node)->chmod(mode);
  }
}

void FilesystemImpl::chown(const boost::filesystem::path &path, uid_t uid, gid_t gid) {
  auto node = _device->Load(path);
  if(node == none) {
    throw fuse::FuseErrnoException(ENOENT);
  } else {
    (*node)->chown(uid, gid);
  }
}

void FilesystemImpl::truncate(const bf::path &path, off_t size) {
  LoadFile(path)->truncate(size);
}

void FilesystemImpl::ftruncate(int descriptor, off_t size) {
  _open_files.get(descriptor)->truncate(size);
}

int FilesystemImpl::read(int descriptor, void *buf, size_t count, off_t offset) {
  return _open_files.get(descriptor)->read(buf, count, offset);
}

void FilesystemImpl::write(int descriptor, const void *buf, size_t count, off_t offset) {
  _open_files.get(descriptor)->write(buf, count, offset);
}

void FilesystemImpl::fsync(int descriptor) {
  _open_files.get(descriptor)->fsync();
}

void FilesystemImpl::fdatasync(int descriptor) {
  _open_files.get(descriptor)->fdatasync();
}

void FilesystemImpl::access(const bf::path &path, int mask) {
  auto node = _device->Load(path);
  if(node == none) {
    throw fuse::FuseErrnoException(ENOENT);
  } else {
    (*node)->access(mask);
  }
}

int FilesystemImpl::createAndOpenFile(const bf::path &path, mode_t mode, uid_t uid, gid_t gid) {
  //TODO Creating the file opens and closes it. We then reopen it afterwards.
  //     This is slow. Improve!
  auto dir = LoadDir(path.parent_path());
  auto file = dir->createAndOpenFile(path.filename().native(), mode, uid, gid);
  return _open_files.open(std::move(file));
}

void FilesystemImpl::mkdir(const bf::path &path, mode_t mode, uid_t uid, gid_t gid) {
  auto dir = LoadDir(path.parent_path());
  dir->createDir(path.filename().native(), mode, uid, gid);
}

void FilesystemImpl::rmdir(const bf::path &path) {
  auto dir = LoadDir(path);
  dir->remove();
}

void FilesystemImpl::unlink(const bf::path &path) {
  auto file = LoadFile(path);
  file->remove();
}

void FilesystemImpl::rename(const bf::path &from, const bf::path &to) {
  auto node = _device->Load(from);
  if(node == none) {
    throw fuse::FuseErrnoException(ENOENT);
  } else {
    (*node)->rename(to);
  }
}

unique_ref<vector<Dir::Entry>> FilesystemImpl::readDir(const bf::path &path) {
  auto dir = LoadDir(path);
  return dir->children();
}

void FilesystemImpl::utimens(const bf::path &path, const timespec times[2]) {
  auto node = _device->Load(path);
  if(node == none) {
    throw fuse::FuseErrnoException(ENOENT);
  } else {
    (*node)->utimens(times);
  }
}

void FilesystemImpl::statfs(const bf::path &path, struct statvfs *fsstat) {
  _device->statfs(path, fsstat);
}

void FilesystemImpl::createSymlink(const bf::path &to, const bf::path &from, uid_t uid, gid_t gid) {
  auto parent = LoadDir(from.parent_path());
  parent->createSymlink(from.filename().native(), to, uid, gid);
}

void FilesystemImpl::readSymlink(const bf::path &path, char *buf, size_t size) {
  string target = LoadSymlink(path)->target().native();
  std::memcpy(buf, target.c_str(), std::min(target.size()+1, size));
  buf[size-1] = '\0';
}
