#include "CryDir.h"

#include <sys/types.h>
#include <sys/stat.h>
#include <fcntl.h>
#include <dirent.h>

#include "fspp/fuse/FuseErrnoException.h"
#include "CryDevice.h"
#include "CryFile.h"

//TODO Get rid of this in favor of exception hierarchy
using fspp::fuse::CHECK_RETVAL;
using fspp::fuse::FuseErrnoException;

namespace bf = boost::filesystem;

using std::unique_ptr;
using std::make_unique;
using std::string;
using std::vector;

namespace cryfs {

CryDir::CryDir() {
}

CryDir::~CryDir() {
}

unique_ptr<fspp::File> CryDir::createFile(const string &name, mode_t mode) {
  throw FuseErrnoException(ENOTSUP);
}

unique_ptr<fspp::Dir> CryDir::createDir(const string &name, mode_t mode) {
  throw FuseErrnoException(ENOTSUP);
}

void CryDir::rmdir() {
  throw FuseErrnoException(ENOTSUP);
}

unique_ptr<vector<string>> CryDir::children() const {
  throw FuseErrnoException(ENOTSUP);
}

}
