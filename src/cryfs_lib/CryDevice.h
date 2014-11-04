#pragma once
#ifndef CRYFS_LIB_CRYDEVICE_H_
#define CRYFS_LIB_CRYDEVICE_H_

#include "fusepp/Fuse.h"

namespace cryfs {

class CryDevice {
public:
	CryDevice();
	virtual ~CryDevice();
};

}

#endif /* CRYFS_LIB_CRYDEVICE_H_ */
