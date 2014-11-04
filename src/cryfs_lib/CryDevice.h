#ifndef CRYFS_LIB_CRYDEVICE_H_
#define CRYFS_LIB_CRYDEVICE_H_

#include "fuse/Fuse.h"

namespace cryfs {

class CryDevice {
public:
	CryDevice();
	virtual ~CryDevice();
private:
	Fuse _fuse;
};

}

#endif /* CRYFS_LIB_CRYDEVICE_H_ */
