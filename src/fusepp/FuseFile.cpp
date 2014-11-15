#include <fusepp/FuseErrnoException.h>
#include <fusepp/FuseFile.h>
#include <fusepp/FuseOpenFile.h>

using std::unique_ptr;
using std::make_unique;

namespace fusepp {

FuseFile::FuseFile(FuseDevice *device, const bf::path &path)
  :FuseNode(device, path) {
  assert(bf::is_regular_file(base_path()));
}

FuseFile::~FuseFile() {
}

std::unique_ptr<FuseOpenFile> FuseFile::open(int flags) const {
  return make_unique<FuseOpenFile>(device(), path(), flags);
}

void FuseFile::truncate(off_t size) const {
  int retval = ::truncate(base_path().c_str(), size);
  CHECK_RETVAL(retval);
}

void FuseFile::unlink() {
  int retval = ::unlink(base_path().c_str());
  CHECK_RETVAL(retval);
}

} /* namespace fusepp */
