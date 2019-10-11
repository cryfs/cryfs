#include "CrySymlink.h"

#include <fspp/fs_interface/FuseErrnoException.h>
#include "CryDevice.h"
#include "CrySymlink.h"
#include "cryfs/impl/filesystem/parallelaccessfsblobstore/SymlinkBlobRef.h"
#include "cryfs/impl/filesystem/fsblobstore/utils/TimestampUpdateBehavior.h"

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

CrySymlink::CrySymlink(CryDevice *device,
                       const BlockId &blockId)
        : CryNode(device, blockId) {
}

CrySymlink::~CrySymlink() {
}

unique_ref<SymlinkBlobRef> CrySymlink::LoadBlob() const {
  auto blob = CryNode::LoadBlob();
  auto symlink_blob = dynamic_pointer_move<SymlinkBlobRef>(blob);
  ASSERT(symlink_blob != none, "Blob does not store a symlink");
  return std::move(*symlink_blob);
}

fspp::Dir::NodeType CrySymlink::getType() const {
  device()->callFsActionCallbacks();
  return fspp::Dir::NodeType::SYMLINK;
}

bf::path CrySymlink::target() {
  device()->callFsActionCallbacks();
  auto blob = LoadBlob(); // NOLINT (workaround https://gcc.gnu.org/bugzilla/show_bug.cgi?id=82481 )
  return blob->target();
}

void CrySymlink::remove() {
  device()->callFsActionCallbacks();
  removeNode();
}

}
