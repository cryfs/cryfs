#pragma once
#ifndef FSPP_IMPL_FILESYSTEMIMPL_H_
#define FSPP_IMPL_FILESYSTEMIMPL_H_

#include "FuseOpenFileList.h"
#include "../fuse/Filesystem.h"

#include <messmer/cpp-utils/pointer/unique_ref.h>

namespace fspp {
class Node;
class File;
class Symlink;
class OpenFile;

class FilesystemImpl: public fuse::Filesystem {
public:
  explicit FilesystemImpl(Device *device);
	virtual ~FilesystemImpl();

	int openFile(const boost::filesystem::path &path, int flags) override;
	void flush(int descriptor) override;
	void closeFile(int descriptor) override;
	void lstat(const boost::filesystem::path &path, struct ::stat *stbuf) override;
	void fstat(int descriptor, struct ::stat *stbuf) override;
	void chmod(const boost::filesystem::path &path, mode_t mode) override;
	void chown(const boost::filesystem::path &path, uid_t uid, gid_t gid) override;
	void truncate(const boost::filesystem::path &path, off_t size) override;
	void ftruncate(int descriptor, off_t size) override;
	int read(int descriptor, void *buf, size_t count, off_t offset) override;
	void write(int descriptor, const void *buf, size_t count, off_t offset) override;
	void fsync(int descriptor) override;
	void fdatasync(int descriptor) override;
	void access(const boost::filesystem::path &path, int mask) override;
	int createAndOpenFile(const boost::filesystem::path &path, mode_t mode, uid_t uid, gid_t gid) override;
	void mkdir(const boost::filesystem::path &path, mode_t mode, uid_t uid, gid_t gid) override;
	void rmdir(const boost::filesystem::path &path) override;
	void unlink(const boost::filesystem::path &path) override;
	void rename(const boost::filesystem::path &from, const boost::filesystem::path &to) override;
	cpputils::unique_ref<std::vector<Dir::Entry>> readDir(const boost::filesystem::path &path) override;
	void utimens(const boost::filesystem::path &path, const timespec times[2]) override;
	void statfs(const boost::filesystem::path &path, struct statvfs *fsstat) override;
    void createSymlink(const boost::filesystem::path &to, const boost::filesystem::path &from, uid_t uid, gid_t gid) override;
    void readSymlink(const boost::filesystem::path &path, char *buf, size_t size) override;

private:
	cpputils::unique_ref<File> LoadFile(const boost::filesystem::path &path);
	cpputils::unique_ref<Dir> LoadDir(const boost::filesystem::path &path);
	cpputils::unique_ref<Symlink> LoadSymlink(const boost::filesystem::path &path);
	int openFile(const File &file, int flags);

	Device *_device;
	FuseOpenFileList _open_files;

  DISALLOW_COPY_AND_ASSIGN(FilesystemImpl);
};

}

#endif /* FSPP_IMPL_FILESYSTEMIMPL_H_ */
