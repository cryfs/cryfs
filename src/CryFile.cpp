#include "CryFile.h"

#include "CryDevice.h"
#include "CryOpenFile.h"
#include "messmer/fspp/fuse/FuseErrnoException.h"
#include "impl/DirBlob.h"

namespace bf = boost::filesystem;

//TODO Get rid of this in favor of exception hierarchy
using fspp::fuse::CHECK_RETVAL;
using fspp::fuse::FuseErrnoException;

using std::unique_ptr;
using std::make_unique;

using blockstore::Key;

namespace cryfs {

CryFile::CryFile(CryDevice *device, unique_ptr<DirBlob> parent, const Key &key)
: CryNode(device, std::move(parent), key) {
}

CryFile::~CryFile() {
}

unique_ptr<fspp::OpenFile> CryFile::open(int flags) const {
  auto blob = LoadBlob();
  assert(blob.get() != nullptr);
  return make_unique<CryOpenFile>(make_unique<FileBlob>(std::move(blob)));
}

void CryFile::truncate(off_t size) const {
  FileBlob(LoadBlob()).resize(size);
}

fspp::Dir::EntryType CryFile::getType() const {
  return fspp::Dir::EntryType::FILE;
}

}
