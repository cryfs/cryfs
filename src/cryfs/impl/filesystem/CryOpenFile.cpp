#include "CryOpenFile.h"

#include <sys/types.h>
#include <fcntl.h>

#include "CryDevice.h"
#include <cpp-utils/logging/logging.h>
#include <cpp-utils/pointer/cast.h>
#include <fspp/fs_interface/FuseErrnoException.h>
#include "entry_helper.h"


using boost::none;
using cpputils::unique_ref;
using cpputils::dynamic_pointer_move;
using cryfs::parallelaccessfsblobstore::FileBlobRef;
using cryfs::parallelaccessfsblobstore::DirBlobRef;
using namespace cpputils::logging;

//TODO Get rid of this in favor of a exception hierarchy

namespace cryfs {

CryOpenFile::CryOpenFile(CryDevice *device, unique_ref<FileBlobRef> fileBlob)
: _device(device), _fileBlob(std::move(fileBlob)) {
}

CryOpenFile::~CryOpenFile() {
  //TODO
} // NOLINT (workaround https://gcc.gnu.org/bugzilla/show_bug.cgi?id=82481 )

unique_ref<DirBlobRef> CryOpenFile::_parentBlob() const {
  auto blob = _device->LoadBlob(_fileBlob->parentPointer());
  auto parent = dynamic_pointer_move<DirBlobRef>(blob);
  if (parent == none) {
    LOG(ERR, "Parent of an open file is not a directory");
    throw fspp::fuse::FuseErrnoException(EIO);
  }
  return std::move(*parent);
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
