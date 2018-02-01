#include "CryNode.h"

#include <sys/time.h>

#include "CryDevice.h"
#include "CryDir.h"
#include "CryFile.h"
#include <fspp/fuse/FuseErrnoException.h>
#include <cpp-utils/pointer/cast.h>
#include <cpp-utils/system/clock_gettime.h>
#include <cpp-utils/system/stat.h>
#include <cpp-utils/logging/logging.h>

namespace bf = boost::filesystem;

using blockstore::BlockId;
using cpputils::unique_ref;
using boost::optional;
using boost::none;
using std::shared_ptr;
using cryfs::fsblobstore::FsBlob;
using cryfs::fsblobstore::DirBlob;
using namespace cpputils::logging;

//TODO Get rid of this in favor of an exception hierarchy
using fspp::fuse::FuseErrnoException;

namespace cryfs {

CryNode::CryNode(CryDevice *device, bf::path path, optional<std::shared_ptr<DirBlob>> parent, optional<std::shared_ptr<DirBlob>> grandparent, const BlockId &blockId)
: _device(device),
  _path(std::move(path)),
  _parent(none),
  _grandparent(none),
  _blockId(blockId) {

  ASSERT(parent != none || grandparent == none, "Grandparent can only be set when parent is not none");

  if (parent != none) {
    _parent = std::move(*parent);
  }
  _grandparent = std::move(grandparent);
}

CryNode::~CryNode() {
}

void CryNode::access(int mask) const {
  // TODO Should we implement access()?
  UNUSED(mask);
  device()->callFsActionCallbacks();
  return;
}

bool CryNode::isRootDir() const {
  return _parent == none;
}

shared_ptr<const DirBlob> CryNode::parent() const {
  ASSERT(_parent != none, "We are the root directory and can't get the parent of the root directory");
  return *_parent;
}

shared_ptr<DirBlob> CryNode::parent() {
  ASSERT(_parent != none, "We are the root directory and can't get the parent of the root directory");
  return *_parent;
}

optional<DirBlob*> CryNode::grandparent() {
  if (_grandparent == none) {
    return none;
  }
  return _grandparent->get();
}

namespace {
// taken from folly (Apache License) https://github.com/facebook/folly/blob/cd1bdc9/folly/experimental/io/FsUtil.cpp
bool skipPrefix(const bf::path& pth, const bf::path& prefix, bf::path::const_iterator& it) {
  it = pth.begin();
  for (auto& p : prefix) {
    if (it == pth.end()) {
      return false;
    }
    if (p == ".") {
      // Should only occur at the end, if prefix ends with a slash
      continue;
    }
    if (*it++ != p) {
      return false;
    }
  }
  return true;
}

// taken from folly (Apache License) https://github.com/facebook/folly/blob/cd1bdc9/folly/experimental/io/FsUtil.cpp
bool starts_with(const bf::path& pth, const bf::path& prefix) {
  bf::path::const_iterator it;
  return skipPrefix(pth, prefix, it);
}

bool path_is_real_prefix(const bf::path& pth, const bf::path& prefix) {
  bf::path::const_iterator it;
  return skipPrefix(pth, prefix, it) && it != pth.end();
}

// taken from folly (Apache License) https://github.com/facebook/folly/blob/cd1bdc9/folly/experimental/io/FsUtil.cpp
bf::path remove_prefix(const bf::path& pth, const bf::path& prefix) {
  bf::path::const_iterator it;
  if (!skipPrefix(pth, prefix, it)) {
    throw std::logic_error(
        "Path does not start with prefix");
  }

  bf::path p;
  for (; it != pth.end(); ++it) {
    p /= *it;
  }

  return p;
}
}


void CryNode::rename(const bf::path &to) {
  // TODO Split into smaller functions
  device()->callFsActionCallbacks();
  ASSERT(_path.empty() || _path.is_absolute(), (string() + "from has to be an absolute path, but is: " + _path.c_str()).c_str());
  ASSERT(to.is_absolute(), "rename target has to be an absolute path. If this assert throws, we have to add code here that makes the path absolute.");
  ASSERT(_path.empty() == (_parent == none), "Path can be empty if and only if we're the root directory");

  if (_parent == none) {
    //We are the root direcory.
    throw FuseErrnoException(EBUSY);
  }

  if (path_is_real_prefix(to, _path)) {
    // Tried to make a dir a subdir of itself
    throw FuseErrnoException(EINVAL);
  }

  // We have to treat cases where the move goes into a subdirectory, the same directory or a sibling directory
  // specially, because we cache the _parent and _grandparent dir blobs in members and (due to locking) can't request
  // them from the blobstore anymore. So use the already loaded _parent and _grandparent blobs instead.
  CryDevice::DirBlobWithParent targetDirWithParent;
  if (path_is_real_prefix(to, _path.parent_path())) {
    // Target is either in same directory (i.e. we're renaming not moving), or in a subdirectory.
    // We can't use normal loading of the target dir starting from the file system root, because that would
    // try to load the parent blob, while it is still stored in the _parent member. This would crash.
    auto relativePath = remove_prefix(to.parent_path(), _path.parent_path());
    targetDirWithParent = _device->LoadDirBlobWithParent(relativePath, *_parent);
  } else if (_grandparent != none && path_is_real_prefix(to, _path.parent_path().parent_path())) {
    // Target is in a sibling directory
    // We can't use normal loading of the target dir starting from the file system root, because that would
    // try to load the grandparent blob, while it is still stored in the _grandparent member. This would crash.
    auto relativePath = remove_prefix(to.parent_path(), _path.parent_path().parent_path());
    targetDirWithParent = _device->LoadDirBlobWithParent(relativePath, *_grandparent);
  } else {
    // Target isn't in the same, sub or sibling directory.
    // We can use normal loading of the target dir starting from the file system root.
    targetDirWithParent = _device->LoadDirBlobWithParent(to.parent_path());
  }

  auto targetDir = std::move(targetDirWithParent.blob);
  auto targetDirParent = std::move(targetDirWithParent.parent);

  auto old = (*_parent)->GetChild(_blockId);
  if (old == boost::none) {
    throw FuseErrnoException(EIO);
  }
  fsblobstore::DirEntry oldEntry = *old; // Copying this (instead of only keeping the reference) is necessary, because the operations below (i.e. RenameChild()) might make a reference invalid.
  auto onOverwritten = [this] (const blockstore::BlockId &blockId) {
      device()->RemoveBlob(blockId);
  };
  _updateParentModificationTimestamp();
  if (targetDir->blockId() == (*_parent)->blockId()) {
    targetDir->RenameChild(oldEntry.blockId(), to.filename().native(), onOverwritten);
  } else {
    _updateTargetDirModificationTimestamp(*targetDir, std::move(targetDirParent));
    targetDir->AddOrOverwriteChild(to.filename().native(), oldEntry.blockId(), oldEntry.type(), oldEntry.mode(), oldEntry.uid(), oldEntry.gid(),
                                   oldEntry.lastAccessTime(), oldEntry.lastModificationTime(), onOverwritten);
    (*_parent)->RemoveChild(oldEntry.name());
    // targetDir is now the new parent for this node. Adapt to it, so we can call further operations on this node object.
    LoadBlob()->setParentPointer(targetDir->blockId());
    _parent = std::move(targetDir);
  }
  _path = to;
}

void CryNode::_updateParentModificationTimestamp() {
  if (_grandparent != none) {
    // TODO Handle timestamps of the root directory (_grandparent == none) correctly.
    ASSERT(_parent != none, "Grandparent is set, so also parent has to be set");
    (*_grandparent)->updateModificationTimestampForChild((*_parent)->blockId());
  }
}

void CryNode::_updateTargetDirModificationTimestamp(const DirBlob &targetDir, optional<shared_ptr<DirBlob>> targetDirParent) {
  if (targetDirParent != none) {
    // TODO Handle timestamps of the root directory (targetDirParent == none) correctly.
    (*targetDirParent)->updateModificationTimestampForChild(targetDir.blockId());
  }
}

void CryNode::utimens(timespec lastAccessTime, timespec lastModificationTime) {
//  LOG(WARN, "---utimens called---");
  device()->callFsActionCallbacks();
  if (_parent == none) {
    //We are the root direcory.
    //TODO What should we do?
    return;
  }
  (*_parent)->utimensChild(_blockId, lastAccessTime, lastModificationTime);
}

void CryNode::removeNode() {
  //TODO Instead of all these if-else and having _parent being an optional, we could also introduce a CryRootDir which inherits from fspp::Dir.
  if (_parent == none) {
    //We are the root direcory.
    //TODO What should we do?
    throw FuseErrnoException(EIO);
  }
  (*_parent)->RemoveChild(_blockId);
  _device->RemoveBlob(_blockId);
}

CryDevice *CryNode::device() {
  return _device;
}

const CryDevice *CryNode::device() const {
  return _device;
}

unique_ref<FsBlob> CryNode::LoadBlob() const {
  auto blob = _device->LoadBlob(_blockId);
  ASSERT(_parent == none || blob->parentPointer() == (*_parent)->blockId(), "Blob has wrong parent pointer.");
  return blob;  // NOLINT (workaround https://gcc.gnu.org/bugzilla/show_bug.cgi?id=82481 )
}

const blockstore::BlockId &CryNode::blockId() const {
  return _blockId;
}

void CryNode::stat(struct ::stat *result) const {
  device()->callFsActionCallbacks();
  if(_parent == none) {
    //We are the root directory.
	  //TODO What should we do?
    result->st_uid = getuid();
    result->st_gid = getgid();
	  result->st_mode = S_IFDIR | S_IRUSR | S_IWUSR | S_IXUSR;
    result->st_size = fsblobstore::DirBlob::DIR_LSTAT_SIZE;
    //TODO If possible without performance loss, then for a directory, st_nlink should return number of dir entries (including "." and "..")
    result->st_nlink = 1;
    struct timespec now{};
    clock_gettime(CLOCK_REALTIME, &now);
    result->st_atim = now;
    result->st_mtim = now;
    result->st_ctim = now;
  } else {
    (*_parent)->statChild(_blockId, result);
  }
}

void CryNode::chmod(mode_t mode) {
  device()->callFsActionCallbacks();
  if (_parent == none) {
    //We are the root direcory.
	//TODO What should we do?
	return;
  }
  (*_parent)->chmodChild(_blockId, mode);
}

void CryNode::chown(uid_t uid, gid_t gid) {
  device()->callFsActionCallbacks();
  if (_parent == none) {
	//We are the root direcory.
	//TODO What should we do?
	return;
  }
  (*_parent)->chownChild(_blockId, uid, gid);
}

bool CryNode::checkParentPointer() {
  auto parentPointer = LoadBlob()->parentPointer();
  if (_parent == none) {
    return parentPointer == BlockId::Null();
  } else {
    return parentPointer == (*_parent)->blockId();
  }
}

}
