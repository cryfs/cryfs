#include "../cryfs_lib/CryDevice.h"

#include <iostream>

using namespace cryfs;

CryDevice::CryDevice(const fusepp::path &rootdir)
  :_rootdir(rootdir) {
}

CryDevice::~CryDevice() {
}
