#include <cryfs_lib/CryOpenFile.h>

#include <sys/types.h>
#include <fcntl.h>

#include "CryErrnoException.h"

using namespace cryfs;

CryOpenFile::CryOpenFile(const bf::path &path, int flags)
  :_descriptor(::open(path.c_str(), flags)) {
  CHECK_RETVAL(_descriptor);
}

CryOpenFile::~CryOpenFile() {
  int retval = close(_descriptor);
  CHECK_RETVAL(retval);
}

void CryOpenFile::stat(struct ::stat *result) const {
  int retval = ::fstat(_descriptor, result);
  CHECK_RETVAL(retval);
}

