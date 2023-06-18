#include "FilesystemImpl.h"

#include <fcntl.h>
#include "../fs_interface/Device.h"
#include "../fs_interface/Dir.h"
#include "../fs_interface/Symlink.h"

#include "../fs_interface/FuseErrnoException.h"
#include "../fs_interface/File.h"
#include "../fs_interface/Node.h"

#include <cpp-utils/logging/logging.h>
#include <cpp-utils/pointer/unique_ref.h>
#include <cpp-utils/system/stat.h>
#include <sstream>

using namespace fspp;
using cpputils::unique_ref;
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

FilesystemImpl::FilesystemImpl(cpputils::unique_ref<Device> device)
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
   _device(std::move(device)), _open_files()
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
  LOG(INFO, profilerInformation.str());
#endif
}

void FilesystemImpl::setContext(Context&& context) {
    _device->setContext(std::move(context));
}

unique_ref<File> FilesystemImpl::LoadFile(const bf::path &path) {
  PROFILE(_loadFileNanosec);
  auto file = _device->LoadFile(path);
  if (file == none) {
    throw fuse::FuseErrnoException(EIO);
  }
  return std::move(*file);
}

unique_ref<Dir> FilesystemImpl::LoadDir(const bf::path &path) {
  PROFILE(_loadDirNanosec);
  auto dir = _device->LoadDir(path);
  if (dir == none) {
    throw fuse::FuseErrnoException(EIO);
  }
  return std::move(*dir);
}

unique_ref<Symlink> FilesystemImpl::LoadSymlink(const bf::path &path) {
  PROFILE(_loadSymlinkNanosec);
  auto lnk = _device->LoadSymlink(path);
  if (lnk == none) {
    throw fuse::FuseErrnoException(EIO);
  }
  return std::move(*lnk);
}

int FilesystemImpl::openFile(const bf::path &path, int flags) {
  auto file = LoadFile(path);
  return openFile(file.get(), flags);
}

int FilesystemImpl::openFile(File *file, int flags) {
  PROFILE(_openFileNanosec);
  return _open_files.open(file->open(fspp::openflags_t(flags)));
}

void FilesystemImpl::flush(int descriptor) {
  PROFILE(_flushNanosec);
  _open_files.load(descriptor, [](OpenFile* openFile) {
	  openFile->flush();
  });
}

void FilesystemImpl::closeFile(int descriptor) {
  PROFILE(_closeFileNanosec);
  _open_files.close(descriptor);
}

namespace {
void convert_stat_info_(const fspp::Node::stat_info& input, fspp::fuse::STAT *output) {
    output->st_nlink = input.nlink;
    output->st_mode = input.mode.value();
    output->st_uid = input.uid.value();
    output->st_gid = input.gid.value();
    output->st_size = input.size.value();
    output->st_blocks = input.blocks;
    output->st_atim = input.atime;
    output->st_mtim = input.mtime;
    output->st_ctim = input.ctime;
}
}

void FilesystemImpl::lstat(const bf::path &path, fspp::fuse::STAT *stbuf) {
  PROFILE(_lstatNanosec);
  auto node = _device->Load(path);
  if(node == none) {
    throw fuse::FuseErrnoException(ENOENT);
  } else {
    auto stat_info = (*node)->stat();
    convert_stat_info_(stat_info, stbuf);
  }
}

void FilesystemImpl::fstat(int descriptor, fspp::fuse::STAT *stbuf) {
	PROFILE(_fstatNanosec);
	auto stat_info = _open_files.load(descriptor, [] (OpenFile* openFile) {
		return openFile->stat();
	});
	convert_stat_info_(stat_info, stbuf);
}

void FilesystemImpl::chmod(const boost::filesystem::path &path, ::mode_t mode) {
  PROFILE(_chmodNanosec);
  auto node = _device->Load(path);
  if(node == none) {
    throw fuse::FuseErrnoException(ENOENT);
  } else {
    (*node)->chmod(fspp::mode_t(mode));
  }
}

void FilesystemImpl::chown(const boost::filesystem::path &path, ::uid_t uid, ::gid_t gid) {
  PROFILE(_chownNanosec);
  auto node = _device->Load(path);
  if(node == none) {
    throw fuse::FuseErrnoException(ENOENT);
  } else {
    (*node)->chown(fspp::uid_t(uid), fspp::gid_t(gid));
  }
}

void FilesystemImpl::truncate(const bf::path &path, fspp::num_bytes_t size) {
  PROFILE(_truncateNanosec);
  LoadFile(path)->truncate(size);
}

void FilesystemImpl::ftruncate(int descriptor, fspp::num_bytes_t size) {
  PROFILE(_ftruncateNanosec);
  _open_files.load(descriptor, [size] (OpenFile* openFile) {
	  openFile->truncate(size);
  });
}

fspp::num_bytes_t FilesystemImpl::read(int descriptor, void *buf, fspp::num_bytes_t count, fspp::num_bytes_t offset) {
  PROFILE(_readNanosec);
  return _open_files.load(descriptor, [buf, count, offset] (OpenFile* openFile) {
	  return openFile->read(buf, count, offset);
  });
}

void FilesystemImpl::write(int descriptor, const void *buf, fspp::num_bytes_t count, fspp::num_bytes_t offset) {
  PROFILE(_writeNanosec);
  return _open_files.load(descriptor, [buf, count, offset] (OpenFile* openFile) {
	  return openFile->write(buf, count, offset);
  });
}

void FilesystemImpl::fsync(int descriptor) {
  PROFILE(_fsyncNanosec);
  _open_files.load(descriptor, [] (OpenFile* openFile) {
	  openFile->fsync();
  });
}

void FilesystemImpl::fdatasync(int descriptor) {
  PROFILE(_fdatasyncNanosec);
  _open_files.load(descriptor, [] (OpenFile* openFile) {
	  openFile->fdatasync();
  });
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

int FilesystemImpl::createAndOpenFile(const bf::path &path, ::mode_t mode, ::uid_t uid, ::gid_t gid) {
  PROFILE(_createAndOpenFileNanosec);
  auto dir = LoadDir(path.parent_path());
  PROFILE(_createAndOpenFileNanosec_withoutLoading);
  auto file = dir->createAndOpenFile(path.filename().string(), fspp::mode_t(mode), fspp::uid_t(uid), fspp::gid_t(gid));
  return _open_files.open(std::move(file));
}

void FilesystemImpl::mkdir(const bf::path &path, ::mode_t mode, ::uid_t uid, ::gid_t gid) {
  PROFILE(_mkdirNanosec);
  auto dir = LoadDir(path.parent_path());
  PROFILE(_mkdirNanosec_withoutLoading);
  dir->createDir(path.filename().string(), fspp::mode_t(mode), fspp::uid_t(uid), fspp::gid_t(gid));
}

void FilesystemImpl::rmdir(const bf::path &path) {
  //TODO Don't allow removing files/symlinks with this
  PROFILE(_rmdirNanosec);
  auto node = _device->Load(path);
  if(node == none) {
    throw fuse::FuseErrnoException(ENOENT);
  }
  PROFILE(_rmdirNanosec_withoutLoading);
  (*node)->remove();
}

void FilesystemImpl::unlink(const bf::path &path) {
  //TODO Don't allow removing directories with this
  PROFILE(_unlinkNanosec);
  auto node = _device->Load(path);
  if (node == none) {
    throw fuse::FuseErrnoException(ENOENT);
  }
  PROFILE(_unlinkNanosec_withoutLoading);
  (*node)->remove();
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

vector<Dir::Entry> FilesystemImpl::readDir(const bf::path &path) {
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

void FilesystemImpl::statfs(struct ::statvfs *fsstat) {
  PROFILE(_statfsNanosec);
  Device::statvfs stat = _device->statfs();

  fsstat->f_bsize = stat.blocksize;
  fsstat->f_blocks = stat.num_total_blocks;
  fsstat->f_bfree = stat.num_free_blocks;
  fsstat->f_bavail = stat.num_available_blocks;
  fsstat->f_files = stat.num_total_inodes;
  fsstat->f_ffree = stat.num_free_inodes;
  fsstat->f_favail = stat.num_available_inodes;
  fsstat->f_namemax = stat.max_filename_length;

  //f_frsize, f_favail, f_fsid and f_flag are ignored in fuse, see http://fuse.sourcearchive.com/documentation/2.7.0/structfuse__operations_4e765e29122e7b6b533dc99849a52655.html#4e765e29122e7b6b533dc99849a52655
  fsstat->f_frsize = fsstat->f_bsize; // even though this is supposed to be ignored, macFUSE needs it.
}

void FilesystemImpl::createSymlink(const bf::path &to, const bf::path &from, ::uid_t uid, ::gid_t gid) {
  PROFILE(_createSymlinkNanosec);
  auto parent = LoadDir(from.parent_path());
  PROFILE(_createSymlinkNanosec_withoutLoading);
  parent->createSymlink(from.filename().string(), to, fspp::uid_t(uid), fspp::gid_t(gid));
}

void FilesystemImpl::readSymlink(const bf::path &path, char *buf, fspp::num_bytes_t size) {
  PROFILE(_readSymlinkNanosec);
  string target = LoadSymlink(path)->target().string();
  PROFILE(_readSymlinkNanosec_withoutLoading);
  std::memcpy(buf, target.c_str(), std::min(static_cast<int64_t>(target.size()+1), size.value()));
  buf[size.value()-1] = '\0';
}
