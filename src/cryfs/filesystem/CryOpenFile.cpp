#include "CryOpenFile.h"

#include <sys/types.h>
#include <fcntl.h>

#include "CryDevice.h"
#include <fspp/fuse/FuseErrnoException.h>

namespace bf = boost::filesystem;

using cpputils::unique_ref;
using cryfs::parallelaccessfsblobstore::FileBlobRef;

//TODO Get rid of this in favor of a exception hierarchy
using fspp::fuse::CHECK_RETVAL;
using fspp::fuse::FuseErrnoException;

namespace cryfs {

CryOpenFile::CryOpenFile(const CryDevice *device, unique_ref<FileBlobRef> fileBlob)
: _device(device), _fileBlob(std::move(fileBlob)) {
}

CryOpenFile::~CryOpenFile() {
  //TODO
}

void CryOpenFile::flush() {
  _device->callFsActionCallbacks();
  _fileBlob->flush();
}

void CryOpenFile::stat(struct ::stat *result) const {
  _device->callFsActionCallbacks();
  result->st_mode = S_IFREG | S_IRUSR | S_IXUSR | S_IWUSR;
  result->st_size = _fileBlob->size();
  return;
}

void CryOpenFile::truncate(off_t size) const {
  _device->callFsActionCallbacks();
  _fileBlob->resize(size);
}

size_t CryOpenFile::read(void *buf, size_t count, off_t offset) const {
  _device->callFsActionCallbacks();
  return _fileBlob->read(buf, offset, count);
}

void CryOpenFile::write(const void *buf, size_t count, off_t offset) {
  _device->callFsActionCallbacks();
  _fileBlob->write(buf, offset, count);
}

void CryOpenFile::fsync() {
  _device->callFsActionCallbacks();
  //throw FuseErrnoException(ENOTSUP);
}

void CryOpenFile::fdatasync() {
  _device->callFsActionCallbacks();
  //throw FuseErrnoException(ENOTSUP);
}

}
