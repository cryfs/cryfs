#include "CryOpenFile.h"

#include <sys/types.h>
#include <fcntl.h>

#include "CryDevice.h"
#include "messmer/fspp/fuse/FuseErrnoException.h"
#include "impl/FileBlob.h"

namespace bf = boost::filesystem;

using std::unique_ptr;
using blobstore::Blob;

//TODO Get rid of this in favor of a exception hierarchy
using fspp::fuse::CHECK_RETVAL;
using fspp::fuse::FuseErrnoException;

namespace cryfs {

CryOpenFile::CryOpenFile(unique_ptr<FileBlob> fileBlob)
: _fileBlob(std::move(fileBlob)) {
}

CryOpenFile::~CryOpenFile() {
  //TODO
}

void CryOpenFile::flush() {
  //throw FuseErrnoException(ENOTSUP);
}

void CryOpenFile::stat(struct ::stat *result) const {
  result->st_mode = S_IFREG | S_IRUSR | S_IXUSR | S_IWUSR;
  result->st_size = _fileBlob->size();
  return;
}

void CryOpenFile::truncate(off_t size) const {
  _fileBlob->resize(size);
}

int CryOpenFile::read(void *buf, size_t count, off_t offset) {
  //TODO Return number of read bytes
  _fileBlob->read(buf, offset, count);
  return count;
}

void CryOpenFile::write(const void *buf, size_t count, off_t offset) {
  _fileBlob->write(buf, offset, count);
}

void CryOpenFile::fsync() {
  //throw FuseErrnoException(ENOTSUP);
}

void CryOpenFile::fdatasync() {
  //throw FuseErrnoException(ENOTSUP);
}

}
