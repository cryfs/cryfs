#include "CryOpenFile.h"

#include <sys/types.h>
#include <fcntl.h>

#include "CryDevice.h"
#include "CryNode.h"
#include <fspp/fs_interface/FuseErrnoException.h>
#include "entry_helper.h"


using std::shared_ptr;
using cpputils::unique_ref;
using cryfs::fsblobstore::rust::RustDirBlob;
using cryfs::fsblobstore::rust::RustFileBlob;
using blockstore::BlockId;

//TODO Get rid of this in favor of a exception hierarchy

namespace cryfs {

CryOpenFile::CryOpenFile(CryDevice *device, const BlockId& parentBlobId, const BlockId& fileBlobId)
: _device(device), _parentBlobId(parentBlobId), _fileBlobId(fileBlobId) {
}

CryOpenFile::~CryOpenFile() {
  //TODO
} // NOLINT (workaround https://gcc.gnu.org/bugzilla/show_bug.cgi?id=82481 )

void CryOpenFile::flush() {
  _device->callFsActionCallbacks();
  LoadFileBlob()->flush();
  LoadParentBlob()->flush();
}

fspp::Node::stat_info CryOpenFile::stat() const {
  _device->callFsActionCallbacks();
  auto childOpt = LoadParentBlob()->GetChild(_fileBlobId);
  if (childOpt == boost::none) {
    throw fspp::fuse::FuseErrnoException(ENOENT);
  }
  return dirEntryToStatInfo(**childOpt, LoadFileBlob()->size());
}

void CryOpenFile::truncate(fspp::num_bytes_t size) const {
  _device->callFsActionCallbacks();
  LoadFileBlob()->resize(size);
  LoadParentBlob()->updateModificationTimestampOfChild(_fileBlobId);
}

fspp::num_bytes_t CryOpenFile::read(void *buf, fspp::num_bytes_t count, fspp::num_bytes_t offset) const {
  _device->callFsActionCallbacks();
  LoadParentBlob()->maybeUpdateAccessTimestampOfChild(_fileBlobId, timestampUpdateBehavior());
  return LoadFileBlob()->read(buf, offset, count);
}

void CryOpenFile::write(const void *buf, fspp::num_bytes_t count, fspp::num_bytes_t offset) {
  _device->callFsActionCallbacks();
  LoadParentBlob()->updateModificationTimestampOfChild(_fileBlobId);
  LoadFileBlob()->write(buf, offset, count);
}

void CryOpenFile::fsync() {
  _device->callFsActionCallbacks();
  LoadFileBlob()->flush();
  LoadParentBlob()->flush();
}

void CryOpenFile::fdatasync() {
  _device->callFsActionCallbacks();
  LoadFileBlob()->flush();
}

fspp::TimestampUpdateBehavior CryOpenFile::timestampUpdateBehavior() const {
  return _device->getContext().timestampUpdateBehavior();
}

unique_ref<RustFileBlob> CryOpenFile::LoadFileBlob() const {
  return std::move(*_device->LoadBlob(_fileBlobId)).asFile();
}

unique_ref<RustDirBlob> CryOpenFile::LoadParentBlob() const {
  return std::move(*_device->LoadBlob(_parentBlobId)).asDir();
}


}
