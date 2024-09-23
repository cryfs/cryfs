#include "CryDir.h"

#include <boost/filesystem/path.hpp>
#include <boost/none.hpp>
#include <cerrno>
#include <cstddef>
#include <string>

#include "CryDevice.h"
#include "CryOpenFile.h"
#include "blockstore/utils/BlockId.h"
#include "cpp-utils/assert/assert.h"
#include "cryfs/impl/filesystem/CryNode.h"
#include "cryfs/impl/filesystem/parallelaccessfsblobstore/DirBlobRef.h"
#include "fspp/fs_interface/Dir.h"
#include "fspp/fs_interface/OpenFile.h"
#include "fspp/fs_interface/Types.h"
#include <cpp-utils/system/time.h>
#include <fspp/fs_interface/FuseErrnoException.h>
#include <utility>

//TODO Get rid of this in favor of exception hierarchy
using fspp::fuse::FuseErrnoException;

namespace bf = boost::filesystem;

using std::string;
using std::vector;

using blockstore::BlockId;
using cpputils::unique_ref;
using cpputils::make_unique_ref;
using cpputils::dynamic_pointer_move;
using boost::optional;
using boost::none;
using cryfs::parallelaccessfsblobstore::DirBlobRef;

namespace cryfs {

CryDir::CryDir(CryDevice *device, optional<unique_ref<DirBlobRef>> parent, optional<unique_ref<DirBlobRef>> grandparent, const BlockId &blockId)
: CryNode(device, std::move(parent), std::move(grandparent), blockId) {
}

CryDir::~CryDir() {
}

unique_ref<fspp::OpenFile> CryDir::createAndOpenFile(const string &name, fspp::mode_t mode, fspp::uid_t uid, fspp::gid_t gid) {
  device()->callFsActionCallbacks();
  if (!isRootDir()) {
    //TODO Instead of doing nothing when we're the root directory, handle timestamps in the root dir correctly (and delete isRootDir() function)
    parent()->updateModificationTimestampForChild(blockId());
  }
  auto child = device()->CreateFileBlob(blockId());
  auto now = cpputils::time::now();
  auto dirBlob = LoadBlob();
  dirBlob->AddChildFile(name, child->blockId(), mode, uid, gid, now, now);
  return make_unique_ref<CryOpenFile>(device(), std::move(dirBlob), std::move(child));
}

void CryDir::createDir(const string &name, fspp::mode_t mode, fspp::uid_t uid, fspp::gid_t gid) {
  device()->callFsActionCallbacks();
  if (!isRootDir()) {
    //TODO Instead of doing nothing when we're the root directory, handle timestamps in the root dir correctly (and delete isRootDir() function)
    parent()->updateModificationTimestampForChild(blockId());
  }
  auto blob = LoadBlob();
  auto child = device()->CreateDirBlob(blockId());
  auto now = cpputils::time::now();
  blob->AddChildDir(name, child->blockId(), mode, uid, gid, now, now);
}

unique_ref<DirBlobRef> CryDir::LoadBlob() const {
  auto blob = CryNode::LoadBlob();
  auto dir_blob = dynamic_pointer_move<DirBlobRef>(blob);
  ASSERT(dir_blob != none, "Blob does not store a directory");
  return std::move(*dir_blob);
}

vector<fspp::Dir::Entry> CryDir::children() {
  device()->callFsActionCallbacks();
  if (!isRootDir()) { // NOLINT (workaround https://gcc.gnu.org/bugzilla/show_bug.cgi?id=82481 )
    //TODO Instead of doing nothing when we're the root directory, handle timestamps in the root dir correctly (and delete isRootDir() function)
    parent()->updateAccessTimestampForChild(blockId(), timestampUpdateBehavior());
  }
  vector<fspp::Dir::Entry> children;
  children.push_back(fspp::Dir::Entry(fspp::Dir::EntryType::DIR, "."));
  children.push_back(fspp::Dir::Entry(fspp::Dir::EntryType::DIR, ".."));
  auto blob = LoadBlob();
  blob->AppendChildrenTo(&children);
  return children;
}

size_t CryDir::numChildren() {
  auto blob = LoadBlob();
  return blob->NumChildren();
}

fspp::Dir::EntryType CryDir::getType() const {
  device()->callFsActionCallbacks();
  return fspp::Dir::EntryType::DIR;
}

void CryDir::createSymlink(const string &name, const bf::path &target, fspp::uid_t uid, fspp::gid_t gid) {
  device()->callFsActionCallbacks();
  if (!isRootDir()) {
    //TODO Instead of doing nothing when we're the root directory, handle timestamps in the root dir correctly (and delete isRootDir() function)
    parent()->updateModificationTimestampForChild(blockId());
  }
  auto blob = LoadBlob();
  auto child = device()->CreateSymlinkBlob(target, blockId());
  auto now = cpputils::time::now();
  blob->AddChildSymlink(name, child->blockId(), uid, gid, now, now);
}

void CryDir::remove() {
  device()->callFsActionCallbacks();
  if (grandparent() != none) {
    //TODO Instead of doing nothing when we're in the root directory, handle timestamps in the root dir correctly
    (*grandparent())->updateModificationTimestampForChild(parent()->blockId());
  }
  {
    auto blob = LoadBlob();
    if (0 != blob->NumChildren()) {
      throw FuseErrnoException(ENOTEMPTY);
    }
  }
  //TODO removeNode() calls CryDevice::RemoveBlob, which loads the blob again. So we're loading it twice. Should be optimized.
  removeNode();
}

}
