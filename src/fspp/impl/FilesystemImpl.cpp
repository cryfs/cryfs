#include "FilesystemImpl.h"

#include <fcntl.h>
#include "../fs_interface/Device.h"
#include "../fs_interface/Dir.h"
#include "../fs_interface/Symlink.h"

#include "../fuse/FuseErrnoException.h"
#include "../fs_interface/File.h"

#include <cpp-utils/logging/logging.h>
#include <cpp-utils/pointer/unique_ref.h>
#include <sstream>

using namespace fspp;
using cpputils::dynamic_pointer_move;
using cpputils::unique_ref;
using cpputils::make_unique_ref;
using std::vector;
using std::string;
using boost::none;

namespace bf = boost::filesystem;
using namespace cpputils::logging;

#ifdef FSPP_PROFILE
#include "Profiler.h"
#include <iomanip>
#include <ios>
#define PROFILE(name) Profiler profiler_##name(&name);
#else
#define PROFILE(name)
#endif

FilesystemImpl::FilesystemImpl(Device *device)
  :
#ifdef FSPP_PROFILE
   _loadFileNanosec(0), _loadDirNanosec(0), _loadSymlinkNanosec(0), _openFileNanosec(0), _flushNanosec(0),
   _closeFileNanosec(0), _lstatNanosec(0), _fstatNanosec(0), _chmodNanosec(0), _chownNanosec(0), _truncateNanosec(0),
   _ftruncateNanosec(0), _readNanosec(0), _writeNanosec(0), _fsyncNanosec(0), _fdatasyncNanosec(0), _accessNanosec(0),
   _createAndOpenFileNanosec(0), _createAndOpenFileNanosec_withoutLoading(0), _mkdirNanosec(0),
   _mkdirNanosec_withoutLoading(0), _rmdirNanosec(0), _rmdirNanosec_withoutLoading(0), _unlinkNanosec(0),
   _unlinkNanosec_withoutLoading(0), _renameNanosec(0), _readDirNanosec(0), _readDirNanosec_withoutLoading(0),
   _utimensNanosec(0), _statfsNanosec(0), _createSymlinkNanosec(0), _createSymlinkNanosec_withoutLoading(0),
   _readSymlinkNanosec(0), _readSymlinkNanosec_withoutLoading(0),
#endif
   _device(device), _open_files()
{
}

FilesystemImpl::~FilesystemImpl() {
#ifdef FSPP_PROFILE
  std::ostringstream profilerInformation;
  profilerInformation << "Profiler Information\n"
    << std::fixed << std::setprecision(6)
    << std::setw(40) << "LoadFile: " << static_cast<double>(_loadFileNanosec)/1000000000 << "\n"
    << std::setw(40) << "LoadDir: " << static_cast<double>(_loadDirNanosec)/1000000000 << "\n"
    << std::setw(40) << "LoadSymlink: " << static_cast<double>(_loadSymlinkNanosec)/1000000000 << "\n"
    << std::setw(40) << "OpenFile: " << static_cast<double>(_openFileNanosec)/1000000000 << "\n"
    << std::setw(40) << "Flush: " << static_cast<double>(_flushNanosec)/1000000000 << "\n"
    << std::setw(40) << "CloseFile: " << static_cast<double>(_closeFileNanosec)/1000000000 << "\n"
    << std::setw(40) << "Lstat: " << static_cast<double>(_lstatNanosec)/1000000000 << "\n"
    << std::setw(40) << "Fstat: " << static_cast<double>(_fstatNanosec)/1000000000 << "\n"
    << std::setw(40) << "Chmod: " << static_cast<double>(_chmodNanosec)/1000000000 << "\n"
    << std::setw(40) << "Chown: " << static_cast<double>(_chownNanosec)/1000000000 << "\n"
    << std::setw(40) << "Truncate: " << static_cast<double>(_truncateNanosec)/1000000000 << "\n"
    << std::setw(40) << "Ftruncate: " << static_cast<double>(_ftruncateNanosec)/1000000000 << "\n"
    << std::setw(40) << "Read: " << static_cast<double>(_readNanosec)/1000000000 << "\n"
    << std::setw(40) << "Write: " << static_cast<double>(_writeNanosec)/1000000000 << "\n"
    << std::setw(40) << "Fsync: " << static_cast<double>(_fsyncNanosec)/1000000000 << "\n"
    << std::setw(40) << "Fdatasync: " << static_cast<double>(_fdatasyncNanosec)/1000000000 << "\n"
    << std::setw(40) << "Access: " << static_cast<double>(_accessNanosec)/1000000000 << "\n"
    << std::setw(40) << "CreateAndOpenFile: " << static_cast<double>(_createAndOpenFileNanosec)/1000000000 << "\n"
    << std::setw(40) << "CreateAndOpenFile (without loading): " << static_cast<double>(_createAndOpenFileNanosec_withoutLoading)/1000000000 << "\n"
    << std::setw(40) << "Mkdir: " << static_cast<double>(_mkdirNanosec)/1000000000 << "\n"
    << std::setw(40) << "Mkdir (without loading): " << static_cast<double>(_mkdirNanosec_withoutLoading)/1000000000 << "\n"
    << std::setw(40) << "Rmdir: " << static_cast<double>(_rmdirNanosec)/1000000000 << "\n"
    << std::setw(40) << "Rmdir (without loading): " << static_cast<double>(_rmdirNanosec_withoutLoading)/1000000000 << "\n"
    << std::setw(40) << "Unlink: " << static_cast<double>(_unlinkNanosec)/1000000000 << "\n"
    << std::setw(40) << "Unlink (without loading): " << static_cast<double>(_unlinkNanosec_withoutLoading)/1000000000 << "\n"
    << std::setw(40) << "Rename: " << static_cast<double>(_renameNanosec)/1000000000 << "\n"
    << std::setw(40) << "ReadDir: " << static_cast<double>(_readDirNanosec)/1000000000 << "\n"
    << std::setw(40) << "ReadDir (without loading): " << static_cast<double>(_readDirNanosec_withoutLoading)/1000000000 << "\n"
    << std::setw(40) << "Utimens: " << static_cast<double>(_utimensNanosec)/1000000000 << "\n"
    << std::setw(40) << "Statfs: " << static_cast<double>(_statfsNanosec)/1000000000 << "\n"
    << std::setw(40) << "CreateSymlink: " << static_cast<double>(_createSymlinkNanosec)/1000000000 << "\n"
    << std::setw(40) << "CreateSymlink (without loading): " << static_cast<double>(_createSymlinkNanosec_withoutLoading)/1000000000 << "\n"
    << std::setw(40) << "ReadSymlink: " << static_cast<double>(_readSymlinkNanosec)/1000000000 << "\n"
    << std::setw(40) << "ReadSymlink (without loading): " << static_cast<double>(_readSymlinkNanosec_withoutLoading)/1000000000 << "\n";
  LOG(INFO) << profilerInformation.str();
#endif
}

unique_ref<File> FilesystemImpl::LoadFile(const bf::path &path) {
  PROFILE(_loadFileNanosec);
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
  PROFILE(_loadDirNanosec);
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
  PROFILE(_loadSymlinkNanosec);
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

unique_ref<Node> FilesystemImpl::LoadFileOrSymlink(const bf::path &path) {
  PROFILE(_loadFileOrSymlinkNanosec);
  auto node = _device->Load(path);
  if (node == none) {
    throw fuse::FuseErrnoException(EIO);
  }
  auto file = dynamic_pointer_move<File>(*node);
  if (file != none) {
    return std::move(*file);
  }

  auto symlink = dynamic_pointer_move<Symlink>(*node);
  if (symlink != none) {
    return std::move(*symlink);
  }

  throw fuse::FuseErrnoException(EISDIR);
}

int FilesystemImpl::openFile(const bf::path &path, int flags) {
  auto file = LoadFile(path);
  return openFile(file.get(), flags);
}

int FilesystemImpl::openFile(File *file, int flags) {
  PROFILE(_openFileNanosec);
  return _open_files.open(file->open(flags));
}

void FilesystemImpl::flush(int descriptor) {
  PROFILE(_flushNanosec);
  _open_files.get(descriptor)->flush();
}

void FilesystemImpl::closeFile(int descriptor) {
  PROFILE(_closeFileNanosec);
  _open_files.close(descriptor);
}

void FilesystemImpl::lstat(const bf::path &path, struct ::stat *stbuf) {
  PROFILE(_lstatNanosec);
  auto node = _device->Load(path);
  if(node == none) {
    throw fuse::FuseErrnoException(ENOENT);
  } else {
    (*node)->stat(stbuf);
  }
}

void FilesystemImpl::fstat(int descriptor, struct ::stat *stbuf) {
  PROFILE(_fstatNanosec);
  _open_files.get(descriptor)->stat(stbuf);
}

void FilesystemImpl::chmod(const boost::filesystem::path &path, mode_t mode) {
  PROFILE(_chmodNanosec);
  auto node = _device->Load(path);
  if(node == none) {
    throw fuse::FuseErrnoException(ENOENT);
  } else {
    (*node)->chmod(mode);
  }
}

void FilesystemImpl::chown(const boost::filesystem::path &path, uid_t uid, gid_t gid) {
  PROFILE(_chownNanosec);
  auto node = _device->Load(path);
  if(node == none) {
    throw fuse::FuseErrnoException(ENOENT);
  } else {
    (*node)->chown(uid, gid);
  }
}

void FilesystemImpl::truncate(const bf::path &path, off_t size) {
  PROFILE(_truncateNanosec);
  LoadFile(path)->truncate(size);
}

void FilesystemImpl::ftruncate(int descriptor, off_t size) {
  PROFILE(_ftruncateNanosec);
  _open_files.get(descriptor)->truncate(size);
}

size_t FilesystemImpl::read(int descriptor, void *buf, size_t count, off_t offset) {
  PROFILE(_readNanosec);
  return _open_files.get(descriptor)->read(buf, count, offset);
}

void FilesystemImpl::write(int descriptor, const void *buf, size_t count, off_t offset) {
  PROFILE(_writeNanosec);
  _open_files.get(descriptor)->write(buf, count, offset);
}

void FilesystemImpl::fsync(int descriptor) {
  PROFILE(_fsyncNanosec);
  _open_files.get(descriptor)->fsync();
}

void FilesystemImpl::fdatasync(int descriptor) {
  PROFILE(_fdatasyncNanosec);
  _open_files.get(descriptor)->fdatasync();
}

void FilesystemImpl::access(const bf::path &path, int mask) {
  PROFILE(_accessNanosec);
  auto node = _device->Load(path);
  if(node == none) {
    throw fuse::FuseErrnoException(ENOENT);
  } else {
    (*node)->access(mask);
  }
}

int FilesystemImpl::createAndOpenFile(const bf::path &path, mode_t mode, uid_t uid, gid_t gid) {
  PROFILE(_createAndOpenFileNanosec);
  auto dir = LoadDir(path.parent_path());
  PROFILE(_createAndOpenFileNanosec_withoutLoading);
  auto file = dir->createAndOpenFile(path.filename().native(), mode, uid, gid);
  return _open_files.open(std::move(file));
}

void FilesystemImpl::mkdir(const bf::path &path, mode_t mode, uid_t uid, gid_t gid) {
  PROFILE(_mkdirNanosec);
  auto dir = LoadDir(path.parent_path());
  PROFILE(_mkdirNanosec_withoutLoading);
  dir->createDir(path.filename().native(), mode, uid, gid);
}

void FilesystemImpl::rmdir(const bf::path &path) {
  PROFILE(_rmdirNanosec);
  auto dir = LoadDir(path);
  PROFILE(_rmdirNanosec_withoutLoading);
  dir->remove();
}

void FilesystemImpl::unlink(const bf::path &path) {
  PROFILE(_unlinkNanosec);
  auto node = LoadFileOrSymlink(path);
  PROFILE(_unlinkNanosec_withoutLoading);
  node->remove();
}

void FilesystemImpl::rename(const bf::path &from, const bf::path &to) {
  PROFILE(_renameNanosec);
  auto node = _device->Load(from);
  if(node == none) {
    throw fuse::FuseErrnoException(ENOENT);
  } else {
    (*node)->rename(to);
  }
}

unique_ref<vector<Dir::Entry>> FilesystemImpl::readDir(const bf::path &path) {
  PROFILE(_readDirNanosec);
  auto dir = LoadDir(path);
  PROFILE(_readDirNanosec_withoutLoading);
  return dir->children();
}

void FilesystemImpl::utimens(const bf::path &path, timespec lastAccessTime, timespec lastModificationTime) {
  PROFILE(_utimensNanosec);
  auto node = _device->Load(path);
  if(node == none) {
    throw fuse::FuseErrnoException(ENOENT);
  } else {
    (*node)->utimens(lastAccessTime, lastModificationTime);
  }
}

void FilesystemImpl::statfs(const bf::path &path, struct statvfs *fsstat) {
  PROFILE(_statfsNanosec);
  _device->statfs(path, fsstat);
}

void FilesystemImpl::createSymlink(const bf::path &to, const bf::path &from, uid_t uid, gid_t gid) {
  PROFILE(_createSymlinkNanosec);
  auto parent = LoadDir(from.parent_path());
  PROFILE(_createSymlinkNanosec_withoutLoading);
  parent->createSymlink(from.filename().native(), to, uid, gid);
}

void FilesystemImpl::readSymlink(const bf::path &path, char *buf, size_t size) {
  PROFILE(_readSymlinkNanosec);
  string target = LoadSymlink(path)->target().native();
  PROFILE(_readSymlinkNanosec_withoutLoading);
  std::memcpy(buf, target.c_str(), std::min(target.size()+1, size));
  buf[size-1] = '\0';
}
