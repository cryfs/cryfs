#pragma once
#ifndef CRYFS_LIB_CRYDEVICE_H_
#define CRYFS_LIB_CRYDEVICE_H_

#include <boost/filesystem.hpp>
#include "utils/macros.h"
#include <memory>

namespace cryfs {
class CryNode;

namespace bf = boost::filesystem;

class CryDevice {
public:
	CryDevice(const bf::path &rootdir);
	virtual ~CryDevice();

	std::unique_ptr<CryNode> LoadFromPath(const bf::path &path);
	//std::unique_ptr<const CryNode> LoadFromPath(const bf::path &path) const;

	const bf::path &RootDir() const;
private:
	const bf::path _rootdir;

  DISALLOW_COPY_AND_ASSIGN(CryDevice);
};

inline const bf::path &CryDevice::RootDir() const {
  return _rootdir;
}

}

#endif /* CRYFS_LIB_CRYDEVICE_H_ */
