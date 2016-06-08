#include "CrySymlink.h"

#include <fspp/fuse/FuseErrnoException.h>
#include "CryDevice.h"
#include "CrySymlink.h"
#include "parallelaccessfsblobstore/SymlinkBlobRef.h"

//TODO Get rid of this in favor of exception hierarchy
using fspp::fuse::CHECK_RETVAL;
using fspp::fuse::FuseErrnoException;

namespace bf = boost::filesystem;

using std::string;
using std::vector;

using blockstore::Key;
using boost::none;
using boost::optional;
using cpputils::unique_ref;
using cpputils::make_unique_ref;
using cpputils::dynamic_pointer_move;
using cryfs::parallelaccessfsblobstore::SymlinkBlobRef;
using cryfs::parallelaccessfsblobstore::DirBlobRef;

namespace cryfs {

CrySymlink::CrySymlink(CryDevice *device, unique_ref<DirBlobRef> parent, optional<unique_ref<DirBlobRef>> grandparent, const Key &key)
: CryNode(device, std::move(parent), std::move(grandparent), key) {
}

CrySymlink::~CrySymlink() {
}

unique_ref<SymlinkBlobRef> CrySymlink::LoadBlob() const {
  auto blob = CryNode::LoadBlob();
  auto symlink_blob = dynamic_pointer_move<SymlinkBlobRef>(blob);
  ASSERT(symlink_blob != none, "Blob does not store a symlink");
  return std::move(*symlink_blob);
}

fspp::Dir::EntryType CrySymlink::getType() const {
  device()->callFsActionCallbacks();
  return fspp::Dir::EntryType::SYMLINK;
}

bf::path CrySymlink::target() {
  device()->callFsActionCallbacks();
  parent()->updateAccessTimestampForChild(key());
  auto blob = LoadBlob();
  return blob->target();
}

void CrySymlink::remove() {
  device()->callFsActionCallbacks();
  if (grandparent() != none) {
    //TODO Instead of doing nothing when we're in the root directory, handle timestamps in the root dir correctly
    (*grandparent())->updateModificationTimestampForChild(parent()->key());
  }
  removeNode();
}

}
