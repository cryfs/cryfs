#include "CryOpenFile.h"

#include <sys/types.h>
#include <fcntl.h>

#include "CryDevice.h"
#include <cpp-utils/logging/logging.h>
#include <cpp-utils/pointer/cast.h>
#include <fspp/fs_interface/FuseErrnoException.h>
#include "entry_helper.h"


using std::shared_ptr;
using boost::none;
using cpputils::unique_ref;
using cpputils::dynamic_pointer_move;
using cryfs::parallelaccessfsblobstore::FileBlobRef;
using cryfs::parallelaccessfsblobstore::DirBlobRef;
using namespace cpputils::logging;

//TODO Get rid of this in favor of a exception hierarchy

namespace cryfs {

CryOpenFile::CryOpenFile(CryDevice *device, shared_ptr<DirBlobRef> parent, unique_ref<FileBlobRef> fileBlob)
: _device(device), _parentMutex(), _parent(std::move(parent)), _fileBlob(std::move(fileBlob)) {
}

CryOpenFile::~CryOpenFile() {
  //TODO
} // NOLINT (workaround https://gcc.gnu.org/bugzilla/show_bug.cgi?id=82481 )

shared_ptr<DirBlobRef> CryOpenFile::_parentBlob() const {
  const std::unique_lock<std::mutex> lock(_parentMutex);
  const blockstore::BlockId &currentParentId = _fileBlob->parentPointer();
  if (currentParentId != _parent->blockId()) {
    // The file was moved into a different directory while it was open. Load the new parent,
    // otherwise we wouldn't find our dir entry anymore.
    auto newParent = _device->LoadBlob(currentParentId);
    auto newParentDir = dynamic_pointer_move<DirBlobRef>(newParent);
    if (newParentDir == none) {
      LOG(ERR, "Parent of an open file is not a directory");
      throw fspp::fuse::FuseErrnoException(EIO);
    }
    _parent = std::move(*newParentDir);
  }
  return _parent;
}

void CryOpenFile::flush() {
  _device->callFsActionCallbacks();
  _fileBlob->flush();
  _parentBlob()->flush();
}

fspp::Node::stat_info CryOpenFile::stat() const {
  _device->callFsActionCallbacks();
  auto parent = _parentBlob();
  auto childOpt = parent->GetChild(_fileBlob->blockId());
  if (childOpt == boost::none) {
    throw fspp::fuse::FuseErrnoException(ENOENT);
  }
  return dirEntryToStatInfo(*childOpt, _fileBlob->size());
}

void CryOpenFile::truncate(fspp::num_bytes_t size) const {
  _device->callFsActionCallbacks();
  _fileBlob->resize(size);
  _parentBlob()->updateModificationTimestampForChild(_fileBlob->blockId());
}

fspp::num_bytes_t CryOpenFile::read(void *buf, fspp::num_bytes_t count, fspp::num_bytes_t offset) const {
  _device->callFsActionCallbacks();
  _parentBlob()->updateAccessTimestampForChild(_fileBlob->blockId(), timestampUpdateBehavior());
  return _fileBlob->read(buf, offset, count);
}

void CryOpenFile::write(const void *buf, fspp::num_bytes_t count, fspp::num_bytes_t offset) {
  _device->callFsActionCallbacks();
  _parentBlob()->updateModificationTimestampForChild(_fileBlob->blockId());
  _fileBlob->write(buf, offset, count);
}

void CryOpenFile::fsync() {
  _device->callFsActionCallbacks();
  _fileBlob->flush();
  _parentBlob()->flush();
}

void CryOpenFile::fdatasync() {
  _device->callFsActionCallbacks();
  _fileBlob->flush();
}

fspp::TimestampUpdateBehavior CryOpenFile::timestampUpdateBehavior() const {
  return _device->getContext().timestampUpdateBehavior();
}

}
