#include "CrySymlink.h"

#include <fspp/fs_interface/FuseErrnoException.h>
#include "CryDevice.h"
#include "CrySymlink.h"

//TODO Get rid of this in favor of exception hierarchy

namespace bf = boost::filesystem;

using std::string;

using blockstore::BlockId;
using boost::none;
using boost::optional;
using cpputils::unique_ref;
using cpputils::dynamic_pointer_move;
using cryfs::fsblobstore::rust::RustSymlinkBlob;
using cryfs::fsblobstore::rust::RustDirBlob;

namespace cryfs {

CrySymlink::CrySymlink(CryDevice *device, const BlockId& parent, optional<blockstore::BlockId> grandparent, const BlockId &blockId)
: CryNode(device, parent, std::move(grandparent), blockId) {
}

CrySymlink::~CrySymlink() {
}

unique_ref<RustSymlinkBlob> CrySymlink::LoadBlob() const {
  return std::move(*CryNode::LoadBlob()).asSymlink();
}

fspp::Dir::EntryType CrySymlink::getType() const {
  device()->callFsActionCallbacks();
  return fspp::Dir::EntryType::SYMLINK;
}

bf::path CrySymlink::target() {
  device()->callFsActionCallbacks();
  LoadParentBlob()->maybeUpdateAccessTimestampOfChild(blockId(), timestampUpdateBehavior());
  auto blob = LoadBlob(); // NOLINT (workaround https://gcc.gnu.org/bugzilla/show_bug.cgi?id=82481 )
  return blob->target();
}

void CrySymlink::remove() {
  device()->callFsActionCallbacks();
  auto grandparent = LoadGrandparentBlobIfHasGrandparent();
  if (grandparent != none) {
    //TODO Instead of doing nothing when we're in the root directory, handle timestamps in the root dir correctly
    (*grandparent)->updateModificationTimestampOfChild(parentBlobId());
  }
  removeNode();
}

}
