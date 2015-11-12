#include "CrySymlink.h"

#include "messmer/fspp/fuse/FuseErrnoException.h"
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

CrySymlink::CrySymlink(CryDevice *device, unique_ref<DirBlobRef> parent, const Key &key)
: CryNode(device, std::move(parent), key) {
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

bf::path CrySymlink::target() const {
  device()->callFsActionCallbacks();
  auto blob = LoadBlob();
  return blob->target();
}

}
