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
using cryfs::parallelaccessfsblobstore::DirBlobRef;
using cryfs::parallelaccessfsblobstore::FileBlobRef;

namespace cryfs {

CryFile::CryFile(CryDevice *device, unique_ref<DirBlobRef> parent, optional<unique_ref<DirBlobRef>> grandparent, const BlockId &blockId)
: CryNode(device, std::move(parent), std::move(grandparent), blockId) {
}

CryFile::~CryFile() {
}

unique_ref<parallelaccessfsblobstore::FileBlobRef> CryFile::LoadBlob() const {
  auto blob = CryNode::LoadBlob();
  auto file_blob = dynamic_pointer_move<FileBlobRef>(blob);
  ASSERT(file_blob != none, "Blob does not store a file");
  return std::move(*file_blob);
}

unique_ref<fspp::OpenFile> CryFile::open(fspp::openflags_t flags) {
  // TODO Should we honor open flags?
  UNUSED(flags);
  device()->callFsActionCallbacks();
  auto blob = LoadBlob();
  return make_unique_ref<CryOpenFile>(device(), parent(), std::move(blob));
}

void CryFile::truncate(fspp::num_bytes_t size) {
  device()->callFsActionCallbacks();
  auto blob = LoadBlob(); // NOLINT (workaround https://gcc.gnu.org/bugzilla/show_bug.cgi?id=82481 )
  blob->resize(size);
  parent()->updateModificationTimestampForChild(blockId());
}

fspp::Dir::EntryType CryFile::getType() const {
  device()->callFsActionCallbacks();
  return fspp::Dir::EntryType::FILE;
}

void CryFile::remove() {
  device()->callFsActionCallbacks();
  if (grandparent() != none) {
    //TODO Instead of doing nothing when we're in the root directory, handle timestamps in the root dir correctly
    (*grandparent())->updateModificationTimestampForChild(parent()->blockId());
  }
  removeNode();
}

}
