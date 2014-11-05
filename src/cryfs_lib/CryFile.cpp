#include "CryFile.h"
#include "CryErrnoException.h"

namespace cryfs {

CryFile::CryFile(CryDevice *device, const bf::path &path)
  :CryNode(device, path) {
}

CryFile::~CryFile() {
}

} /* namespace cryfs */
