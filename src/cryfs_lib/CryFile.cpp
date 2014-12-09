#include "CryFile.h"

#include "CryDevice.h"
#include "CryOpenFile.h"
#include "fspp/fuse/FuseErrnoException.h"

namespace bf = boost::filesystem;

//TODO Get rid of this in favor of exception hierarchy
using fspp::fuse::CHECK_RETVAL;
using fspp::fuse::FuseErrnoException;

using std::unique_ptr;
using std::make_unique;

namespace cryfs {

CryFile::CryFile(unique_ptr<FileBlock> block)
: _block(std::move(block)) {
}

CryFile::~CryFile() {
}

unique_ptr<fspp::OpenFile> CryFile::open(int flags) const {
  throw FuseErrnoException(ENOTSUP);
}

void CryFile::truncate(off_t size) const {
  throw FuseErrnoException(ENOTSUP);
}

void CryFile::unlink() {
  throw FuseErrnoException(ENOTSUP);
}

}
