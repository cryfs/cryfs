#include "CryFile.h"

#include "CryDevice.h"
#include "CryOpenFile.h"
#include <fspp/fs_interface/FuseErrnoException.h>


//TODO Get rid of this in favor of exception hierarchy

using blockstore::BlockId;
using boost::none;
using boost::optional;
using cpputils::unique_ref;
using cpputils::make_unique_ref;
using cpputils::dynamic_pointer_move;
using cryfs::fsblobstore::rust::RustDirBlob;
using cryfs::fsblobstore::rust::RustFileBlob;

namespace cryfs {

CryFile::CryFile(CryDevice *device, const BlockId& parentBlobId, optional<BlockId> grandparentBlobId, const BlockId &blockId)
: CryNode(device, parentBlobId, std::move(grandparentBlobId), blockId) {
}

CryFile::~CryFile() {
}

unique_ref<RustFileBlob> CryFile::LoadBlob() const {
  return std::move(*CryNode::LoadBlob()).asFile();
}

unique_ref<fspp::OpenFile> CryFile::open(fspp::openflags_t flags) {
  // TODO Should we honor open flags?
  UNUSED(flags);
  device()->callFsActionCallbacks();
  return make_unique_ref<CryOpenFile>(device(), parentBlobId(), blockId());
}

void CryFile::truncate(fspp::num_bytes_t size) {
  device()->callFsActionCallbacks();
  auto blob = LoadBlob(); // NOLINT (workaround https://gcc.gnu.org/bugzilla/show_bug.cgi?id=82481 )
  blob->resize(size);
  LoadParentBlob()->updateModificationTimestampOfChild(blockId());
}

fspp::Dir::EntryType CryFile::getType() const {
  device()->callFsActionCallbacks();
  return fspp::Dir::EntryType::FILE;
}

void CryFile::remove() {
  device()->callFsActionCallbacks();
  auto grandparent = LoadGrandparentBlobIfHasGrandparent();
  if (grandparent != none) {
    //TODO Instead of doing nothing when we're in the root directory, handle timestamps in the root dir correctly
    (*grandparent)->updateModificationTimestampOfChild(parentBlobId());
  }
  removeNode();
}

}
