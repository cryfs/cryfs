#include "CrySymlink.h"

#include <fspp/fuse/FuseErrnoException.h>
#include "CryDevice.h"
#include "CrySymlink.h"
#include "fsblobstore/SymlinkBlob.h"
#include "fsblobstore/utils/TimestampUpdateBehavior.h"

//TODO Get rid of this in favor of exception hierarchy

namespace bf = boost::filesystem;

using std::string;

using blockstore::BlockId;
using boost::none;
using boost::optional;
using cpputils::unique_ref;
using cpputils::dynamic_pointer_move;
using cryfs::fsblobstore::SymlinkBlob;
using cryfs::fsblobstore::DirBlob;

namespace cryfs {

CrySymlink::CrySymlink(CryDevice *device, bf::path path, std::shared_ptr<DirBlob> parent, optional<std::shared_ptr<DirBlob>> grandparent, const BlockId &blockId)
: CryNode(device, std::move(path), std::move(parent), std::move(grandparent), blockId) {
}

CrySymlink::~CrySymlink() {
}

unique_ref<SymlinkBlob> CrySymlink::LoadBlob() const {
  auto blob = CryNode::LoadBlob();
  auto symlink_blob = dynamic_pointer_move<SymlinkBlob>(blob);
  ASSERT(symlink_blob != none, "Blob does not store a symlink");
  return std::move(*symlink_blob);
}

fspp::Dir::EntryType CrySymlink::getType() const {
  device()->callFsActionCallbacks();
  return fspp::Dir::EntryType::SYMLINK;
}

bf::path CrySymlink::target() {
  device()->callFsActionCallbacks();
  parent()->updateAccessTimestampForChild(blockId(), fsblobstore::TimestampUpdateBehavior::RELATIME);
  auto blob = LoadBlob(); // NOLINT (workaround https://gcc.gnu.org/bugzilla/show_bug.cgi?id=82481 )
  return blob->target();
}

void CrySymlink::remove() {
  device()->callFsActionCallbacks();
  if (grandparent() != none) {
    //TODO Instead of doing nothing when we're in the root directory, handle timestamps in the root dir correctly
    (*grandparent())->updateModificationTimestampForChild(parent()->blockId());
  }
  removeNode();
}

}
