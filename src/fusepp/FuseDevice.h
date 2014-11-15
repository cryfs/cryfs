#pragma once
#ifndef FUSEPP_FUSEDEVICE_H_
#define FUSEPP_FUSEDEVICE_H_

#include <boost/filesystem.hpp>
#include <fusepp/FuseOpenFileList.h>
#include <memory>
#include <sys/stat.h>
#include <sys/statvfs.h>

#include "utils/macros.h"

namespace fusepp {
class FuseNode;
class FuseFile;
class FuseOpenFile;
class FuseDir;

class FuseDevice {
public:
	FuseDevice();
	virtual ~FuseDevice();

	int openFile(const boost::filesystem::path &path, int flags);
	void closeFile(int descriptor);
	void lstat(const boost::filesystem::path &path, struct ::stat *stbuf);
	void fstat(int descriptor, struct ::stat *stbuf);
	void truncate(const boost::filesystem::path &path, off_t size);
	void ftruncate(int descriptor, off_t size);
	int read(int descriptor, void *buf, size_t count, off_t offset);
	void write(int descriptor, const void *buf, size_t count, off_t offset);
	void fsync(int descriptor);
	void fdatasync(int descriptor);
	void access(const boost::filesystem::path &path, int mask);
	int createAndOpenFile(const boost::filesystem::path &path, mode_t mode);
	void mkdir(const boost::filesystem::path &path, mode_t mode);
	void rmdir(const boost::filesystem::path &path);
	void unlink(const boost::filesystem::path &path);
	void rename(const boost::filesystem::path &from, const boost::filesystem::path &to);
	std::unique_ptr<std::vector<std::string>> readDir(const boost::filesystem::path &path);
	void utimens(const boost::filesystem::path &path, const timespec times[2]);
	virtual void statfs(const boost::filesystem::path &path, struct statvfs *fsstat) = 0;

private:
	virtual std::unique_ptr<FuseNode> Load(const boost::filesystem::path &path) = 0;
	std::unique_ptr<FuseFile> LoadFile(const boost::filesystem::path &path);
	std::unique_ptr<FuseDir> LoadDir(const boost::filesystem::path &path);
	int openFile(const FuseFile &file, int flags);
	FuseOpenFileList _open_files;

  DISALLOW_COPY_AND_ASSIGN(FuseDevice);
};

}

#endif /* FUSEPP_FUSEDEVICE_H_ */
