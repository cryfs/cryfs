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

CryOpenFile::CryOpenFile(CryDevice *device, blockstore::BlockId parentBlobId, blockstore::BlockId fileBlobId, std::weak_ptr<fsblobstore::DirBlob> parent)
: _device(device), _parentBlobId(parentBlobId), _fileBlobId(fileBlobId), _parent(std::move(parent)) {
}

CryOpenFile::~CryOpenFile() {
  //TODO
} // NOLINT (workaround https://gcc.gnu.org/bugzilla/show_bug.cgi?id=82481 )

void CryOpenFile::flush() {
  _device->callFsActionCallbacks();
}

void CryOpenFile::stat(struct ::stat *result) const {
  _device->callFsActionCallbacks();
  result->st_size = _Load()->size();
  _LoadParent()->statChildWithSizeAlreadySet(_fileBlobId, result);
}

void CryOpenFile::truncate(off_t size) {
  _device->callFsActionCallbacks();
  _Load()->resize(size);
  _LoadParent()->updateModificationTimestampForChild(_fileBlobId);
}

size_t CryOpenFile::read(void *buf, size_t count, off_t offset) const {
  _device->callFsActionCallbacks();
  _LoadParent()->updateAccessTimestampForChild(_fileBlobId, fsblobstore::TimestampUpdateBehavior::RELATIME);
  return _Load()->read(buf, offset, count);
}

void CryOpenFile::write(const void *buf, size_t count, off_t offset) {
  _device->callFsActionCallbacks();
  _LoadParent()->updateModificationTimestampForChild(_fileBlobId);
  _Load()->write(buf, offset, count);
}

void CryOpenFile::fsync() {
  _device->callFsActionCallbacks();
}

void CryOpenFile::fdatasync() {
  _device->callFsActionCallbacks();
}

unique_ref<fsblobstore::FileBlob> CryOpenFile::_Load() const {
  auto blob = _device->LoadBlob(_fileBlobId);
  auto fileBlob = cpputils::dynamic_pointer_move<fsblobstore::FileBlob>(blob);
  if (fileBlob == boost::none) {
    throw std::runtime_error("Blob for open file is not a file blob");
  }
  return std::move(*fileBlob);
}

shared_ptr<fsblobstore::DirBlob> CryOpenFile::_LoadParent() const {
  if (auto parent = _parent.lock()) {
    return std::move(parent);
  } else {
    auto blob = _device->LoadBlob(_parentBlobId);
    auto dirBlob = cpputils::dynamic_pointer_move<fsblobstore::DirBlob>(blob);
    if (dirBlob == boost::none) {
      throw std::runtime_error("Blob for parent dir of open file is not a dir");
    }
    return std::move(*dirBlob);
  }
}

}
