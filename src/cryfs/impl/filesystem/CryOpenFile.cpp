#include "CryOpenFile.h"

#include "CryDevice.h"
#include <fspp/fs_interface/FuseErrnoException.h>


using cpputils::unique_ref;
using cryfs::parallelaccessfsblobstore::FileBlobRef;

//TODO Get rid of this in favor of a exception hierarchy

namespace cryfs {

CryOpenFile::CryOpenFile(const CryDevice *device, unique_ref<FileBlobRef> fileBlob)
: _device(device),  _fileBlob(std::move(fileBlob)) {
}

CryOpenFile::~CryOpenFile() = default;

void CryOpenFile::flush() {
  _device->callFsActionCallbacks();
  _fileBlob->flush();
}

fspp::Node::stat_info CryOpenFile::stat() const {
  _device->callFsActionCallbacks();
  return _fileBlob->metaData()._info;
}

void CryOpenFile::truncate(fspp::num_bytes_t size) const {
  _device->callFsActionCallbacks();
  _fileBlob->resize(size);
}

fspp::num_bytes_t CryOpenFile::read(void *buf, fspp::num_bytes_t count, fspp::num_bytes_t offset) const {
  _device->callFsActionCallbacks();
  return _fileBlob->read(buf, offset, count);
}

void CryOpenFile::write(const void *buf, fspp::num_bytes_t count, fspp::num_bytes_t offset) {
  _device->callFsActionCallbacks();
  _fileBlob->write(buf, offset, count);
}

void CryOpenFile::fsync() {
  _device->callFsActionCallbacks();
  _fileBlob->flush();
}

void CryOpenFile::fdatasync() {
  _device->callFsActionCallbacks();
  _fileBlob->flush();
}

}
