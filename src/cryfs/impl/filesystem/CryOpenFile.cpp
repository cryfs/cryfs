#include "CryOpenFile.h"

#include <sys/types.h>
#include <fcntl.h>

#include "CryDevice.h"
#include <fspp/fs_interface/FuseErrnoException.h>


using std::shared_ptr;
using cpputils::unique_ref;
using cryfs::parallelaccessfsblobstore::FileBlobRef;
using cryfs::parallelaccessfsblobstore::DirBlobRef;

//TODO Get rid of this in favor of a exception hierarchy

namespace cryfs {

CryOpenFile::CryOpenFile(const CryDevice *device, shared_ptr<DirBlobRef> parent, unique_ref<FileBlobRef> fileBlob)
: _device(device), _parent(parent), _fileBlob(std::move(fileBlob)) {
}

CryOpenFile::~CryOpenFile() {
  //TODO
} // NOLINT (workaround https://gcc.gnu.org/bugzilla/show_bug.cgi?id=82481 )

void CryOpenFile::flush() {
  _device->callFsActionCallbacks();
  _fileBlob->flush();
  _parent->flush();
}

fspp::Node::stat_info CryOpenFile::stat() const {
  _device->callFsActionCallbacks();
  return _parent->statChildWithKnownSize(_fileBlob->blockId(), _fileBlob->size());
}

void CryOpenFile::truncate(fspp::num_bytes_t size) const {
  _device->callFsActionCallbacks();
  _fileBlob->resize(size);
  _parent->updateModificationTimestampForChild(_fileBlob->blockId());
}

fspp::num_bytes_t CryOpenFile::read(void *buf, fspp::num_bytes_t count, fspp::num_bytes_t offset) const {
  _device->callFsActionCallbacks();
  _parent->updateAccessTimestampForChild(_fileBlob->blockId(), timestampUpdateBehavior());
  return _fileBlob->read(buf, offset, count);
}

void CryOpenFile::write(const void *buf, fspp::num_bytes_t count, fspp::num_bytes_t offset) {
  _device->callFsActionCallbacks();
  _parent->updateModificationTimestampForChild(_fileBlob->blockId());
  _fileBlob->write(buf, offset, count);
}

void CryOpenFile::fsync() {
  _device->callFsActionCallbacks();
  _fileBlob->flush();
  _parent->flush();
}

void CryOpenFile::fdatasync() {
  _device->callFsActionCallbacks();
  _fileBlob->flush();
}

fspp::TimestampUpdateBehavior CryOpenFile::timestampUpdateBehavior() const {
  return _device->getContext().timestampUpdateBehavior();
}

}
