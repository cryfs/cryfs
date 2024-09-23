#include "CrySymlink.h"

#include "CryDevice.h"
#include "CrySymlink.h"
#include "blockstore/utils/BlockId.h"
#include "cpp-utils/assert/assert.h"
#include "cryfs/impl/filesystem/CryNode.h"
#include "cryfs/impl/filesystem/parallelaccessfsblobstore/DirBlobRef.h"
#include "cryfs/impl/filesystem/parallelaccessfsblobstore/SymlinkBlobRef.h"
#include "fspp/fs_interface/Dir.h"
#include <boost/filesystem/path.hpp>
#include <boost/none.hpp>
#include <string>
#include <utility>

//TODO Get rid of this in favor of exception hierarchy

namespace bf = boost::filesystem;

using std::string;

using blockstore::BlockId;
using boost::none;
using boost::optional;
using cpputils::unique_ref;
using cpputils::dynamic_pointer_move;
using cryfs::parallelaccessfsblobstore::SymlinkBlobRef;
using cryfs::parallelaccessfsblobstore::DirBlobRef;

namespace cryfs {

CrySymlink::CrySymlink(CryDevice *device, unique_ref<DirBlobRef> parent, optional<unique_ref<DirBlobRef>> grandparent, const BlockId &blockId)
: CryNode(device, std::move(parent), std::move(grandparent), blockId) {
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
  parent()->updateAccessTimestampForChild(blockId(), timestampUpdateBehavior());
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
