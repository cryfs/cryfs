#include "CryOpenFile.h"

#include <sys/types.h>
#include <fcntl.h>

#include "CryDevice.h"
#include <fspp/fuse/FuseErrnoException.h>


using std::shared_ptr;
using cpputils::unique_ref;
using cryfs::fsblobstore::FileBlob;
using cryfs::fsblobstore::DirBlob;

//TODO Get rid of this in favor of a exception hierarchy

namespace cryfs {

CryOpenFile::CryOpenFile(const CryDevice *device, shared_ptr<DirBlob> parent, unique_ref<FileBlob> fileBlob)
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

void CryOpenFile::stat(struct ::stat *result) const {
  _device->callFsActionCallbacks();
  result->st_size = _fileBlob->size();
  _parent->statChildWithSizeAlreadySet(_fileBlob->blockId(), result);
}

void CryOpenFile::truncate(off_t size) const {
  _device->callFsActionCallbacks();
  _fileBlob->resize(size);
  _parent->updateModificationTimestampForChild(_fileBlob->blockId());
}

size_t CryOpenFile::read(void *buf, size_t count, off_t offset) const {
  _device->callFsActionCallbacks();
  _parent->updateAccessTimestampForChild(_fileBlob->blockId(), fsblobstore::TimestampUpdateBehavior::RELATIME);
  return _fileBlob->read(buf, offset, count);
}

void CryOpenFile::write(const void *buf, size_t count, off_t offset) {
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

}
