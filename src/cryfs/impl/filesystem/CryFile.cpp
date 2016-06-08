#include "CryFile.h"

#include "CryDevice.h"
#include "CryOpenFile.h"
#include <fspp/fuse/FuseErrnoException.h>

namespace bf = boost::filesystem;

//TODO Get rid of this in favor of exception hierarchy
using fspp::fuse::CHECK_RETVAL;
using fspp::fuse::FuseErrnoException;

using blockstore::Key;
using boost::none;
using boost::optional;
using cpputils::unique_ref;
using cpputils::make_unique_ref;
using cpputils::dynamic_pointer_move;
using cryfs::parallelaccessfsblobstore::DirBlobRef;
using cryfs::parallelaccessfsblobstore::FileBlobRef;

namespace cryfs {

CryFile::CryFile(CryDevice *device, unique_ref<DirBlobRef> parent, optional<unique_ref<DirBlobRef>> grandparent, const Key &key)
: CryNode(device, std::move(parent), std::move(grandparent), key) {
}

CryFile::~CryFile() {
}

unique_ref<parallelaccessfsblobstore::FileBlobRef> CryFile::LoadBlob() const {
  auto blob = CryNode::LoadBlob();
  auto file_blob = dynamic_pointer_move<FileBlobRef>(blob);
  ASSERT(file_blob != none, "Blob does not store a file");
  return std::move(*file_blob);
}

unique_ref<fspp::OpenFile> CryFile::open(int flags) {
  // TODO Should we honor open flags?
  UNUSED(flags);
  device()->callFsActionCallbacks();
  auto blob = LoadBlob();
  return make_unique_ref<CryOpenFile>(device(), parent(), std::move(blob));
}

void CryFile::truncate(off_t size) {
  device()->callFsActionCallbacks();
  auto blob = LoadBlob();
  blob->resize(size);
  parent()->updateModificationTimestampForChild(key());
}

fspp::Dir::EntryType CryFile::getType() const {
  device()->callFsActionCallbacks();
  return fspp::Dir::EntryType::FILE;
}

void CryFile::remove() {
  device()->callFsActionCallbacks();
  if (grandparent() != none) {
    //TODO Instead of doing nothing when we're in the root directory, handle timestamps in the root dir correctly
    (*grandparent())->updateModificationTimestampForChild(parent()->key());
  }
  removeNode();
}

}
