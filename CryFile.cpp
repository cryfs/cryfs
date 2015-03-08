#include "CryFile.h"

#include "CryDevice.h"
#include "CryOpenFile.h"
#include "messmer/fspp/fuse/FuseErrnoException.h"

namespace bf = boost::filesystem;

//TODO Get rid of this in favor of exception hierarchy
using fspp::fuse::CHECK_RETVAL;
using fspp::fuse::FuseErrnoException;

using std::unique_ptr;
using std::make_unique;

namespace cryfs {

CryFile::CryFile(CryDevice *device, unique_ptr<FileBlob> blob)
: _device(device),
  _blob(std::move(blob)) {
}

CryFile::~CryFile() {
}

unique_ptr<fspp::OpenFile> CryFile::open(int flags) const {
  //TODO This is a performance issue because we open the blob twice on the "open" syscall
  return make_unique<CryOpenFile>(make_unique<FileBlob>(_device->LoadBlob(_blob->key())));
}

void CryFile::stat(struct ::stat *result) const {
  result->st_mode = S_IFREG | S_IRUSR | S_IXUSR | S_IWUSR;
  return;
  throw FuseErrnoException(ENOTSUP);
}

void CryFile::truncate(off_t size) const {
  throw FuseErrnoException(ENOTSUP);
}

void CryFile::unlink() {
  throw FuseErrnoException(ENOTSUP);
}

}
