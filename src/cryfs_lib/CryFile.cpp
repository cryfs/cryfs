#include "CryFile.h"
#include "CryOpenFile.h"

using std::unique_ptr;
using std::make_unique;

namespace cryfs {

CryFile::CryFile(CryDevice *device, const bf::path &path)
  :CryNode(device, path) {
}

CryFile::~CryFile() {
}

std::unique_ptr<CryOpenFile> CryFile::open(int flags) const {
  return make_unique<CryOpenFile>(base_path(), flags);
}

} /* namespace cryfs */
