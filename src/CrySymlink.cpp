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
using boost::none;

namespace cryfs {

CrySymlink::CrySymlink(CryDevice *device, unique_ptr<DirBlob> parent, const Key &key)
: CryNode(device, std::move(parent), key) {
}

CrySymlink::~CrySymlink() {
}

unique_ptr<SymlinkBlob> CrySymlink::LoadBlob() const {
  auto blob = CryNode::LoadBlob();
  if (blob == none) {
    return nullptr;
  }
  return make_unique<SymlinkBlob>(std::move(*blob));
}

fspp::Dir::EntryType CrySymlink::getType() const {
  return fspp::Dir::EntryType::SYMLINK;
}

bf::path CrySymlink::target() const {
  return LoadBlob()->target();
}

}
