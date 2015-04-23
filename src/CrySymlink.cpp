#include "CrySymlink.h"

#include "messmer/fspp/fuse/FuseErrnoException.h"
#include "CryDevice.h"
#include "CrySymlink.h"
#include "impl/SymlinkBlob.h"

//TODO Get rid of this in favor of exception hierarchy
using fspp::fuse::CHECK_RETVAL;
using fspp::fuse::FuseErrnoException;

namespace bf = boost::filesystem;

using std::unique_ptr;
using std::make_unique;
using std::string;
using std::vector;

using blockstore::Key;

namespace cryfs {

CrySymlink::CrySymlink(CryDevice *device, unique_ptr<DirBlob> parent, const Key &key)
: CryNode(device, std::move(parent), key) {
}

CrySymlink::~CrySymlink() {
}

unique_ptr<SymlinkBlob> CrySymlink::LoadBlob() const {
  return make_unique<SymlinkBlob>(CryNode::LoadBlob());
}

fspp::Dir::EntryType CrySymlink::getType() const {
  return fspp::Dir::EntryType::SYMLINK;
}

bf::path CrySymlink::target() const {
  return LoadBlob()->target();
}

}
