#include "CrySymlink.h"

#include "messmer/fspp/fuse/FuseErrnoException.h"
#include "CryDevice.h"
#include "CrySymlink.h"
#include "fsblobstore/SymlinkBlob.h"

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
using cryfs::fsblobstore::SymlinkBlob;
using cryfs::fsblobstore::DirBlob;

namespace cryfs {

CrySymlink::CrySymlink(CryDevice *device, unique_ref<DirBlob> parent, const Key &key)
: CryNode(device, std::move(parent), key) {
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
  return fspp::Dir::EntryType::SYMLINK;
}

bf::path CrySymlink::target() const {
  auto blob = LoadBlob();
  return blob->target();
}

}
