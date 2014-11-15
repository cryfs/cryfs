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

namespace bf = boost::filesystem;

class FuseDevice {
public:
	FuseDevice(const bf::path &rootdir);
	virtual ~FuseDevice();

	int openFile(const bf::path &path, int flags);
	void closeFile(int descriptor);
	void lstat(const bf::path &path, struct ::stat *stbuf);
	void fstat(int descriptor, struct ::stat *stbuf);
	void truncate(const bf::path &path, off_t size);
	void ftruncate(int descriptor, off_t size);
	int read(int descriptor, void *buf, size_t count, off_t offset);
	void write(int descriptor, const void *buf, size_t count, off_t offset);
	void fsync(int descriptor);
	void fdatasync(int descriptor);
	void access(const bf::path &path, int mask);
	int createAndOpenFile(const bf::path &path, mode_t mode);
	void mkdir(const bf::path &path, mode_t mode);
	void rmdir(const bf::path &path);
	void unlink(const bf::path &path);
	void rename(const bf::path &from, const bf::path &to);
	std::unique_ptr<std::vector<std::string>> readDir(const bf::path &path);
	void utimens(const bf::path &path, const timespec times[2]);
	void statfs(const bf::path &path, struct statvfs *fsstat);

	const bf::path &RootDir() const;
private:
	std::unique_ptr<FuseNode> Load(const bf::path &path);
	std::unique_ptr<FuseFile> LoadFile(const bf::path &path);
	std::unique_ptr<FuseDir> LoadDir(const bf::path &path);
	int openFile(const FuseFile &file, int flags);
	const bf::path _rootdir;
	FuseOpenFileList _open_files;

  DISALLOW_COPY_AND_ASSIGN(FuseDevice);
};

inline const bf::path &FuseDevice::RootDir() const {
  return _rootdir;
}

}

#endif /* FUSEPP_FUSEDEVICE_H_ */
