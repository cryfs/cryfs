#include "CryDir.h"

#include <sys/types.h>
#include <sys/stat.h>
#include <fcntl.h>
#include <dirent.h>

#include <fspp/fuse/FuseErrnoException.h>
#include "CryDevice.h"
#include "CryFile.h"
#include "CryOpenFile.h"
#include <cpp-utils/system/time.h>

//TODO Get rid of this in favor of exception hierarchy
using fspp::fuse::CHECK_RETVAL;
using fspp::fuse::FuseErrnoException;

namespace bf = boost::filesystem;

using std::string;
using std::vector;

using blockstore::Key;
using cpputils::unique_ref;
using cpputils::make_unique_ref;
using cpputils::dynamic_pointer_move;
using boost::optional;
using boost::none;
using cryfs::parallelaccessfsblobstore::DirBlobRef;

namespace cryfs {

CryDir::CryDir(CryDevice *device, optional<unique_ref<DirBlobRef>> parent, optional<unique_ref<DirBlobRef>> grandparent, const Key &key)
: CryNode(device, std::move(parent), std::move(grandparent), key) {
}

CryDir::~CryDir() {
}

unique_ref<fspp::OpenFile> CryDir::createAndOpenFile(const string &name, mode_t mode, uid_t uid, gid_t gid) {
  device()->callFsActionCallbacks();
  if (!isRootDir()) {
    //TODO Instead of doing nothing when we're the root directory, handle timestamps in the root dir correctly (and delete isRootDir() function)
    parent()->updateModificationTimestampForChild(key());
  }
  auto child = device()->CreateFileBlob();
  auto now = cpputils::time::now();
  auto dirBlob = LoadBlob();
  dirBlob->AddChildFile(name, child->key(), mode, uid, gid, now, now);
  return make_unique_ref<CryOpenFile>(device(), cpputils::to_unique_ptr(std::move(dirBlob)), std::move(child));
}

void CryDir::createDir(const string &name, mode_t mode, uid_t uid, gid_t gid) {
  device()->callFsActionCallbacks();
  if (!isRootDir()) {
    //TODO Instead of doing nothing when we're the root directory, handle timestamps in the root dir correctly (and delete isRootDir() function)
    parent()->updateModificationTimestampForChild(key());
  }
  auto blob = LoadBlob();
  auto child = device()->CreateDirBlob();
  auto now = cpputils::time::now();
  blob->AddChildDir(name, child->key(), mode, uid, gid, now, now);
}

unique_ref<DirBlobRef> CryDir::LoadBlob() const {
  auto blob = CryNode::LoadBlob();
  auto dir_blob = dynamic_pointer_move<DirBlobRef>(blob);
  ASSERT(dir_blob != none, "Blob does not store a directory");
  return std::move(*dir_blob);
}

unique_ref<vector<fspp::Dir::Entry>> CryDir::children() {
  device()->callFsActionCallbacks();
  if (!isRootDir()) {
    //TODO Instead of doing nothing when we're the root directory, handle timestamps in the root dir correctly (and delete isRootDir() function)
    parent()->updateAccessTimestampForChild(key());
  }
  auto children = make_unique_ref<vector<fspp::Dir::Entry>>();
  children->push_back(fspp::Dir::Entry(fspp::Dir::EntryType::DIR, "."));
  children->push_back(fspp::Dir::Entry(fspp::Dir::EntryType::DIR, ".."));
  auto blob = LoadBlob();
  blob->AppendChildrenTo(children.get());
  return children;
}

fspp::Dir::EntryType CryDir::getType() const {
  device()->callFsActionCallbacks();
  return fspp::Dir::EntryType::DIR;
}

void CryDir::createSymlink(const string &name, const bf::path &target, uid_t uid, gid_t gid) {
  device()->callFsActionCallbacks();
  if (!isRootDir()) {
    //TODO Instead of doing nothing when we're the root directory, handle timestamps in the root dir correctly (and delete isRootDir() function)
    parent()->updateModificationTimestampForChild(key());
  }
  auto blob = LoadBlob();
  auto child = device()->CreateSymlinkBlob(target);
  auto now = cpputils::time::now();
  blob->AddChildSymlink(name, child->key(), uid, gid, now, now);
}

void CryDir::remove() {
  device()->callFsActionCallbacks();
  if (grandparent() != none) {
    //TODO Instead of doing nothing when we're in the root directory, handle timestamps in the root dir correctly
    (*grandparent())->updateModificationTimestampForChild(parent()->key());
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
