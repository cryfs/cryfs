#pragma once
#ifndef FSPP_IMPL_FILESYSTEMIMPL_H_
#define FSPP_IMPL_FILESYSTEMIMPL_H_

#include "FuseOpenFileList.h"
#include "Filesystem.h"

#include "fspp/utils/macros.h"

namespace fspp {
class Node;
class File;
class OpenFile;
class Dir;

class FilesystemImpl: public Filesystem {
public:
  FilesystemImpl(Device *device);
	virtual ~FilesystemImpl();

	int openFile(const boost::filesystem::path &path, int flags) override;
	void closeFile(int descriptor) override;
	void lstat(const boost::filesystem::path &path, struct ::stat *stbuf) override;
	void fstat(int descriptor, struct ::stat *stbuf) override;
	void truncate(const boost::filesystem::path &path, off_t size) override;
	void ftruncate(int descriptor, off_t size) override;
	int read(int descriptor, void *buf, size_t count, off_t offset) override;
	void write(int descriptor, const void *buf, size_t count, off_t offset) override;
	void fsync(int descriptor) override;
	void fdatasync(int descriptor) override;
	void access(const boost::filesystem::path &path, int mask) override;
	int createAndOpenFile(const boost::filesystem::path &path, mode_t mode) override;
	void mkdir(const boost::filesystem::path &path, mode_t mode) override;
	void rmdir(const boost::filesystem::path &path) override;
	void unlink(const boost::filesystem::path &path) override;
	void rename(const boost::filesystem::path &from, const boost::filesystem::path &to) override;
	std::unique_ptr<std::vector<std::string>> readDir(const boost::filesystem::path &path) override;
	void utimens(const boost::filesystem::path &path, const timespec times[2]) override;
	void statfs(const boost::filesystem::path &path, struct statvfs *fsstat) override;

private:
	std::unique_ptr<File> LoadFile(const boost::filesystem::path &path);
	std::unique_ptr<Dir> LoadDir(const boost::filesystem::path &path);
	int openFile(const File &file, int flags);

	Device *_device;
	FuseOpenFileList _open_files;

  DISALLOW_COPY_AND_ASSIGN(FilesystemImpl);
};

}

#endif /* FSPP_IMPL_FILESYSTEMIMPL_H_ */
