#include "CryFile.h"
#include "CryOpenFile.h"

#include "CryErrnoException.h"

using std::unique_ptr;
using std::make_unique;

namespace cryfs {

CryFile::CryFile(CryDevice *device, const bf::path &path)
  :CryNode(device, path) {
  assert(bf::is_regular_file(base_path()));
}

CryFile::~CryFile() {
}

std::unique_ptr<CryOpenFile> CryFile::open(int flags) const {
  return make_unique<CryOpenFile>(base_path(), flags);
}

void CryFile::truncate(off_t size) const {
  int retval = ::truncate(base_path().c_str(), size);
  CHECK_RETVAL(retval);
}

void CryFile::unlink() {
  int retval = ::unlink(base_path().c_str());
  CHECK_RETVAL(retval);
}

} /* namespace cryfs */
