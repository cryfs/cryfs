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

CrySymlink::CrySymlink(CryDevice *device, unique_ref<RustDirBlob> parent, optional<unique_ref<RustDirBlob>> grandparent, const BlockId &blockId)
: CryNode(device, std::move(parent), std::move(grandparent), blockId) {
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
  parent()->maybeUpdateAccessTimestampOfChild(blockId(), timestampUpdateBehavior());
  auto blob = LoadBlob(); // NOLINT (workaround https://gcc.gnu.org/bugzilla/show_bug.cgi?id=82481 )
  return blob->target();
}

void CrySymlink::remove() {
  device()->callFsActionCallbacks();
  if (grandparent() != none) {
    //TODO Instead of doing nothing when we're in the root directory, handle timestamps in the root dir correctly
    (*grandparent())->updateModificationTimestampOfChild(parent()->blockId());
  }
  removeNode();
}

}
