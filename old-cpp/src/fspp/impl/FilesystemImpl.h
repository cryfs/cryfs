#pragma once
#ifndef MESSMER_FSPP_IMPL_FILESYSTEMIMPL_H_
#define MESSMER_FSPP_IMPL_FILESYSTEMIMPL_H_

#include "FuseOpenFileList.h"
#include "../fuse/Filesystem.h"

#include <cpp-utils/pointer/unique_ref.h>
#include <atomic>

//Remove this line if you don't want profiling
//#define FSPP_PROFILE 1

//TODO Test

namespace fspp {
class Node;
class File;
class Symlink;
class OpenFile;

class FilesystemImpl final: public fuse::Filesystem {
public:
  explicit FilesystemImpl(cpputils::unique_ref<Device> device);
	virtual ~FilesystemImpl();

    void setContext(Context&& context) override;

	int openFile(const boost::filesystem::path &path, int flags) override;
	void flush(int descriptor) override;
	void closeFile(int descriptor) override;
	void lstat(const boost::filesystem::path &path, fspp::fuse::STAT *stbuf) override;
	void fstat(int descriptor, fspp::fuse::STAT *stbuf) override;
	void chmod(const boost::filesystem::path &path, ::mode_t mode) override;
	void chown(const boost::filesystem::path &path, ::uid_t uid, ::gid_t gid) override;
	void truncate(const boost::filesystem::path &path, fspp::num_bytes_t size) override;
	void ftruncate(int descriptor, fspp::num_bytes_t size) override;
	fspp::num_bytes_t read(int descriptor, void *buf, fspp::num_bytes_t count, fspp::num_bytes_t offset) override;
	void write(int descriptor, const void *buf, fspp::num_bytes_t count, fspp::num_bytes_t offset) override;
	void fsync(int descriptor) override;
	void fdatasync(int descriptor) override;
	void access(const boost::filesystem::path &path, int mask) override;
	int createAndOpenFile(const boost::filesystem::path &path, ::mode_t mode, ::uid_t uid, ::gid_t gid) override;
	void mkdir(const boost::filesystem::path &path, ::mode_t mode, ::uid_t uid, ::gid_t gid) override;
	void rmdir(const boost::filesystem::path &path) override;
	void unlink(const boost::filesystem::path &path) override;
	void rename(const boost::filesystem::path &from, const boost::filesystem::path &to) override;
	std::vector<Dir::Entry> readDir(const boost::filesystem::path &path) override;
	void utimens(const boost::filesystem::path &path, timespec lastAccessTime, timespec lastModificationTime) override;
	void statfs(struct ::statvfs *fsstat) override;
    void createSymlink(const boost::filesystem::path &to, const boost::filesystem::path &from, ::uid_t uid, ::gid_t gid) override;
    void readSymlink(const boost::filesystem::path &path, char *buf, fspp::num_bytes_t size) override;

private:
	cpputils::unique_ref<File> LoadFile(const boost::filesystem::path &path);
	cpputils::unique_ref<Dir> LoadDir(const boost::filesystem::path &path);
	cpputils::unique_ref<Symlink> LoadSymlink(const boost::filesystem::path &path);
	int openFile(File *file, int flags);

#ifdef FSPP_PROFILE
    std::atomic<uint64_t> _loadFileNanosec;
    std::atomic<uint64_t> _loadDirNanosec;
    std::atomic<uint64_t> _loadSymlinkNanosec;
    std::atomic<uint64_t> _loadFileOrSymlinkNanosec;
    std::atomic<uint64_t> _openFileNanosec;
    std::atomic<uint64_t> _flushNanosec;
    std::atomic<uint64_t> _closeFileNanosec;
    std::atomic<uint64_t> _lstatNanosec;
    std::atomic<uint64_t> _fstatNanosec;
    std::atomic<uint64_t> _chmodNanosec;
    std::atomic<uint64_t> _chownNanosec;
    std::atomic<uint64_t> _truncateNanosec;
    std::atomic<uint64_t> _ftruncateNanosec;
    std::atomic<uint64_t> _readNanosec;
    std::atomic<uint64_t> _writeNanosec;
    std::atomic<uint64_t> _fsyncNanosec;
    std::atomic<uint64_t> _fdatasyncNanosec;
    std::atomic<uint64_t> _accessNanosec;
    std::atomic<uint64_t> _createAndOpenFileNanosec;
    std::atomic<uint64_t> _createAndOpenFileNanosec_withoutLoading;
    std::atomic<uint64_t> _mkdirNanosec;
    std::atomic<uint64_t> _mkdirNanosec_withoutLoading;
    std::atomic<uint64_t> _rmdirNanosec;
    std::atomic<uint64_t> _rmdirNanosec_withoutLoading;
    std::atomic<uint64_t> _unlinkNanosec;
    std::atomic<uint64_t> _unlinkNanosec_withoutLoading;
    std::atomic<uint64_t> _renameNanosec;
    std::atomic<uint64_t> _readDirNanosec;
    std::atomic<uint64_t> _readDirNanosec_withoutLoading;
    std::atomic<uint64_t> _utimensNanosec;
    std::atomic<uint64_t> _statfsNanosec;
    std::atomic<uint64_t> _createSymlinkNanosec;
    std::atomic<uint64_t> _createSymlinkNanosec_withoutLoading;
    std::atomic<uint64_t> _readSymlinkNanosec;
    std::atomic<uint64_t> _readSymlinkNanosec_withoutLoading;
#endif

	cpputils::unique_ref<Device> _device;
	FuseOpenFileList _open_files;

  DISALLOW_COPY_AND_ASSIGN(FilesystemImpl);
};

}

#endif
