#pragma once
#ifndef CRYFS_LIB_CRYDEVICE_H_
#define CRYFS_LIB_CRYDEVICE_H_

#include "fusepp/Fuse.h"
#include "utils/macros.h"

namespace cryfs {

class CryDevice {
public:
	CryDevice(const fusepp::path &rootdir);
	virtual ~CryDevice();

	const fusepp::path &RootDir() const;
private:
	const fusepp::path _rootdir;

  DISALLOW_COPY_AND_ASSIGN(CryDevice);
};

inline const fusepp::path &CryDevice::RootDir() const {
  return _rootdir;
}

}

#endif /* CRYFS_LIB_CRYDEVICE_H_ */
