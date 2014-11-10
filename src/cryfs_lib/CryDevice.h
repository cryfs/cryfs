#pragma once
#ifndef CRYFS_LIB_CRYDEVICE_H_
#define CRYFS_LIB_CRYDEVICE_H_

#include <boost/filesystem.hpp>
#include <cryfs_lib/CryOpenFileList.h>
#include <memory>
#include <sys/stat.h>

#include "utils/macros.h"

namespace cryfs {
class CryNode;
class CryFile;

namespace bf = boost::filesystem;

class CryDevice {
public:
	CryDevice(const bf::path &rootdir);
	virtual ~CryDevice();

	int OpenFile(const bf::path &path, int flags);
	void lstat(const bf::path &path, struct ::stat *stbuf);
	void fstat(int descriptor, struct ::stat *stbuf);

	const bf::path &RootDir() const;
private:
	std::unique_ptr<CryNode> Load(const bf::path &path);
	std::unique_ptr<CryFile> LoadFile(const bf::path &path);
	const bf::path _rootdir;
	CryOpenFileList _open_files;

  DISALLOW_COPY_AND_ASSIGN(CryDevice);
};

inline const bf::path &CryDevice::RootDir() const {
  return _rootdir;
}

}

#endif /* CRYFS_LIB_CRYDEVICE_H_ */
