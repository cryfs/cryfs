#include "CryNode.h"

#include "CryDevice.h"
#include "CryDir.h"
#include "CryFile.h"
#include <fspp/fs_interface/FuseErrnoException.h>
#include <cpp-utils/pointer/cast.h>
#include <cpp-utils/system/time.h>
#include <cpp-utils/system/stat.h>
#include <cpp-utils/logging/logging.h>
#include "entry_helper.h"

namespace bf = boost::filesystem;

using blockstore::BlockId;
using cpputils::unique_ref;
using cpputils::dynamic_pointer_move;
using boost::optional;
using boost::none;
using std::shared_ptr;
using cryfs::fsblobstore::rust::RustDirBlob;
using cryfs::fsblobstore::rust::RustFsBlob;
using namespace cpputils::logging;

//TODO Get rid of this in favor of an exception hierarchy
using fspp::fuse::FuseErrnoException;

namespace cryfs {

CryNode::CryNode(CryDevice *device, optional<BlockId> parentBlobId, optional<BlockId> grandparentBlobId, const BlockId &blockId)
: _device(device),
  _parentBlobId(std::move(parentBlobId)),
  _grandparentBlobId(std::move(grandparentBlobId)),
  _blockId(blockId) {

  ASSERT(_parentBlobId != none || _grandparentBlobId == none, "Grandparent can only be set when parent is not none");
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
  return _parentBlobId == none;
}

const blockstore::BlockId& CryNode::parentBlobId() const {
  ASSERT(_parentBlobId != none, "Can't load parent blob of root directory");
  return *_parentBlobId;
}

unique_ref<RustDirBlob> CryNode::LoadParentBlob() const {
  ASSERT(_parentBlobId != none, "Can't load parent blob of root directory");
  return std::move(*_device->LoadBlob(*_parentBlobId)).asDir();
}

optional<unique_ref<RustDirBlob>> CryNode::LoadGrandparentBlobIfHasGrandparent() const {
  if (_grandparentBlobId == none) {
    return none;
  }
  return std::move(*_device->LoadBlob(*_grandparentBlobId)).asDir();
}

fspp::TimestampUpdateBehavior CryNode::timestampUpdateBehavior() const {
  return _device->getContext().timestampUpdateBehavior();
}

void CryNode::rename(const bf::path &to) {
  device()->callFsActionCallbacks();
  if (_parentBlobId == none) {
    //We are the root direcory.
    throw FuseErrnoException(EBUSY);
  }
  if (!to.has_parent_path()) {
    // Target is the root directory
    throw FuseErrnoException(EBUSY);
  }

  auto targetParentAndAncestors = _device->LoadDirBlobWithAncestors(to.parent_path(), [&] (const BlockId& ancestorId) {
    if (ancestorId == _blockId) {
      // We are trying to move a node into one of its subdirectories. This is not allowed.
      throw FuseErrnoException(EINVAL);
    }
  });
  if (targetParentAndAncestors == none) {
    // Target parent directory doesn't exist
    throw FuseErrnoException(ENOENT);
  }
  shared_ptr<RustDirBlob> targetParent = std::move(targetParentAndAncestors->blob);
  optional<shared_ptr<RustDirBlob>> targetGrandparent =
    targetParentAndAncestors->parent == none ? none : optional<shared_ptr<RustDirBlob>>(std::move(*targetParentAndAncestors->parent));
  if (targetParent->blockId() == _blockId) {
    // We are trying to move a node into one of its subdirectories. This is not allowed.
    throw FuseErrnoException(EINVAL);
  }

  // TODO There's a deadlock opportunity here because we lock first targetParent+targetGrandparent and then our parent.
  //      If another thread locks our parent first and then targetParent+targetGrandparent, we deadlock.

  // Load parent blob but in a way that doesn't deadlock if the parent blob
  // is already loaded as targetParent or targetGrandparent
  shared_ptr<RustDirBlob> parent;
  if (_parentBlobId == targetParent->blockId()) {
    parent = targetParent;
  } else if (targetGrandparent != none && _parentBlobId == (*targetGrandparent)->blockId()) {
    parent = *targetGrandparent;
  } else {
    parent = LoadParentBlob();
  }

  auto old = parent->GetChild(_blockId);
  if (old == boost::none) {
    throw FuseErrnoException(EIO);
  }
  unique_ref<fsblobstore::rust::RustDirEntry> oldEntry = std::move(*old);
  auto onOverwritten = [this] (const blockstore::BlockId &blockId) {
      device()->RemoveBlob(blockId);
  };
  if (targetParent->blockId() == _parentBlobId) {
    targetParent->RenameChild(oldEntry->blockId(), to.filename().string(), onOverwritten);
    targetGrandparent = none; // destruct so that we free its lock for loads further down and don't deadlock
    _updateParentModificationTimestamp();
  } else {
    auto preexistingTargetEntry = targetParent->GetChild(to.filename().string());
    if (preexistingTargetEntry != boost::none && (*preexistingTargetEntry)->type() == fspp::Dir::EntryType::DIR) {
      if (getType() != fspp::Dir::EntryType::DIR) {
        // A directory cannot be overwritten with a non-directory
        throw FuseErrnoException(EISDIR);
      }
      if ((*preexistingTargetEntry)->blockId() == _parentBlobId) {
        // We are trying to make a node into its parent. This is not allowed.
        throw FuseErrnoException(ENOTEMPTY);
      }
      auto preexistingTarget = device()->LoadBlob((*preexistingTargetEntry)->blockId());
      if (!preexistingTarget->isDir()) {
        LOG(ERR, "Preexisting target is not a directory. But its parent dir entry says it's a directory");
        throw FuseErrnoException(EIO);
      }
      auto preexistingTargetDir = std::move(*preexistingTarget).asDir();
      if (preexistingTargetDir->NumChildren() > 0) {
        // Cannot overwrite a non-empty dir with a rename operation.
        throw FuseErrnoException(ENOTEMPTY);
      }
    }

    _updateTargetDirModificationTimestamp(*targetParent, targetGrandparent);
    targetGrandparent = none; // destruct so that we free its lock for loads further down and don't deadlock
    targetParent->AddOrOverwriteChild(to.filename().string(), oldEntry->blockId(), oldEntry->type(), oldEntry->mode(), oldEntry->uid(), oldEntry->gid(),
                                      oldEntry->lastAccessTime(), oldEntry->lastModificationTime(), onOverwritten);
    parent->RemoveChild(oldEntry->name());
    // targetParent is now the new parent for this node. Adapt to it, so we can call further operations on this node object.
    auto newParentBlobId = targetParent->blockId();
    targetParent.reset(); // destruct so that we free its lock for loads further down and don't deadlock
    _updateParentModificationTimestamp();
    LoadBlob()->setParent(newParentBlobId);
    _parentBlobId = newParentBlobId;
  }
}

void CryNode::_updateParentModificationTimestamp() {
  if (_grandparentBlobId != none) {
    // TODO Handle timestamps of the root directory (_grandparentBlobId == none) correctly.
    ASSERT(_parentBlobId != none, "Grandparent is set, so also parent has to be set");
    (*LoadGrandparentBlobIfHasGrandparent())->updateModificationTimestampOfChild(*_parentBlobId);
  }
}

void CryNode::_updateTargetDirModificationTimestamp(const RustDirBlob &targetDir, optional<shared_ptr<RustDirBlob>> targetDirParent) {
  if (targetDirParent != none) {
    // TODO Handle timestamps of the root directory (targetDirParent == none) correctly.
    (*targetDirParent)->updateModificationTimestampOfChild(targetDir.blockId());
  }
}

void CryNode::utimens(timespec lastAccessTime, timespec lastModificationTime) {
//  LOG(WARN, "---utimens called---");
  device()->callFsActionCallbacks();
  if (_parentBlobId == none) {
    //We are the root direcory.
    //TODO What should we do?
    return;
  }
  LoadParentBlob()->setAccessTimesOfChild(_blockId, lastAccessTime, lastModificationTime);
}

void CryNode::removeNode() {
  //TODO Instead of all these if-else and having _parent being an optional, we could also introduce a CryRootDir which inherits from fspp::Dir.
  if (_parentBlobId == none) {
    //We are the root direcory.
    //TODO What should we do?
    throw FuseErrnoException(EIO);
  }
  LoadParentBlob()->RemoveChildIfExists(_blockId);
  _device->RemoveBlob(_blockId);
}

CryDevice *CryNode::device() {
  return _device;
}

const CryDevice *CryNode::device() const {
  return _device;
}

unique_ref<RustFsBlob> CryNode::LoadBlob() const {
  auto blob = _device->LoadBlob(_blockId);
  ASSERT(_parentBlobId == none || blob->parent() == _parentBlobId, "Blob has wrong parent pointer.");
  return blob;  // NOLINT (workaround https://gcc.gnu.org/bugzilla/show_bug.cgi?id=82481 )
}

const blockstore::BlockId &CryNode::blockId() const {
  return _blockId;
}

CryNode::stat_info CryNode::stat() const {
  device()->callFsActionCallbacks();
  if(_parentBlobId == none) {
    stat_info result;
    //We are the root directory.
    //TODO What should we do?
#if defined(_MSC_VER)
    // TODO And what to do on Windows?
    result.uid = fspp::uid_t(1000);
    result.gid = fspp::gid_t(1000);
#else
    result.uid = fspp::uid_t(getuid());
    result.gid = fspp::gid_t(getgid());
#endif
    result.mode = fspp::mode_t().addDirFlag().addUserReadFlag().addUserWriteFlag().addUserExecFlag();
    result.size = fsblobstore::DirBlob::DIR_LSTAT_SIZE;
    //TODO If possible without performance loss, then for a directory, st_nlink should return number of dir entries (including "." and "..")
    result.nlink = 1;
    struct timespec now = cpputils::time::now();
    result.atime = now;
    result.mtime = now;
    result.ctime = now;
    return result;
  } else {
    auto childOpt = LoadParentBlob()->GetChild(_blockId);
    if (childOpt == boost::none) {
      throw fspp::fuse::FuseErrnoException(ENOENT);
    }
    return dirEntryToStatInfo(**childOpt, LoadBlob()->lstat_size());
  }
}

void CryNode::chmod(fspp::mode_t mode) {
  device()->callFsActionCallbacks();
  if (_parentBlobId == none) {
    //We are the root direcory.
    //TODO What should we do?
    return;
  }
  LoadParentBlob()->setModeOfChild(_blockId, mode);
}

void CryNode::chown(fspp::uid_t uid, fspp::gid_t gid) {
  device()->callFsActionCallbacks();
  if (_parentBlobId == none) {
	//We are the root direcory.
	//TODO What should we do?
	return;
  }
  LoadParentBlob()->setUidGidOfChild(_blockId, uid, gid);
}

bool CryNode::checkParentPointer() {
  auto parentPointer = LoadBlob()->parent();
  if (_parentBlobId == none) {
    return parentPointer == BlockId::Null();
  } else {
    return parentPointer == _parentBlobId;
  }
}

}
