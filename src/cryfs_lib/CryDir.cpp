#include "CryDir.h"
#include "CryDevice.h"

namespace cryfs {

CryDir::CryDir(CryDevice *device, const bf::path &path)
  :CryNode(device, path) {
}

CryDir::~CryDir() {
}

} /* namespace cryfs */
