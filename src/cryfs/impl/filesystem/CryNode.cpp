#include "CryNode.h"

#include "CryDevice.h"
#include "blockstore/utils/BlockId.h"
#include "cpp-utils/assert/assert.h"
#include "cpp-utils/macros.h"
#include "cryfs/impl/filesystem/fsblobstore/DirBlob.h"
#include "cryfs/impl/filesystem/fsblobstore/utils/DirEntry.h"
#include "cryfs/impl/filesystem/parallelaccessfsblobstore/DirBlobRef.h"
#include "cryfs/impl/filesystem/parallelaccessfsblobstore/FsBlobRef.h"
#include "entry_helper.h"
#include "fspp/fs_interface/Context.h"
#include "fspp/fs_interface/Dir.h"
#include "fspp/fs_interface/Types.h"
#include <boost/filesystem/path.hpp>
#include <boost/none.hpp>
#include <cerrno>
#include <cpp-utils/logging/logging.h>
#include <cpp-utils/pointer/cast.h>
#include <cpp-utils/system/time.h>
#include <ctime>
#include <fspp/fs_interface/FuseErrnoException.h>
#include <unistd.h>
#include <utility>

namespace bf = boost::filesystem;

using blockstore::BlockId;
using cpputils::unique_ref;
using cpputils::dynamic_pointer_move;
using boost::optional;
using boost::none;
using std::shared_ptr;
using cryfs::parallelaccessfsblobstore::FsBlobRef;
using cryfs::parallelaccessfsblobstore::DirBlobRef;
using namespace cpputils::logging;

//TODO Get rid of this in favor of an exception hierarchy
using fspp::fuse::FuseErrnoException;

namespace cryfs {

CryNode::CryNode(CryDevice *device, optional<unique_ref<DirBlobRef>> parent, optional<unique_ref<DirBlobRef>> grandparent, const BlockId &blockId)
: _device(device),
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

shared_ptr<const DirBlobRef> CryNode::parent() const {
  ASSERT(_parent != none, "We are the root directory and can't get the parent of the root directory");
  return *_parent;
}

shared_ptr<DirBlobRef> CryNode::parent() {
  ASSERT(_parent != none, "We are the root directory and can't get the parent of the root directory");
  return *_parent;
}

optional<DirBlobRef*> CryNode::grandparent() {
  if (_grandparent == none) {
    return none;
  }
  return _grandparent->get();
}

fspp::TimestampUpdateBehavior CryNode::timestampUpdateBehavior() const {
  return _device->getContext().timestampUpdateBehavior();
}

void CryNode::rename(const bf::path &to) {
  device()->callFsActionCallbacks();
  if (_parent == none) {
    // We are the root direcory.
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
  auto targetParent = std::move(targetParentAndAncestors->blob);
  auto targetGrandparent = std::move(targetParentAndAncestors->parent);
  if (targetParent->blockId() == _blockId) {
    // We are trying to move a node into one of its subdirectories. This is not allowed.
      throw FuseErrnoException(EINVAL);
  }

  auto old = (*_parent)->GetChild(_blockId);
  if (old == boost::none) {
    throw FuseErrnoException(EIO);
  }
  const fsblobstore::DirEntry oldEntry = *old; // Copying this (instead of only keeping the reference) is necessary, because the operations below (i.e. RenameChild()) might make a reference invalid.
  auto onOverwritten = [this] (const blockstore::BlockId &blockId) {
      device()->RemoveBlob(blockId);
  };
  if (targetParent->blockId() == (*_parent)->blockId()) {
    _updateParentModificationTimestamp();
    targetParent->RenameChild(oldEntry.blockId(), to.filename().string(), onOverwritten);
  } else {
    auto preexistingTargetEntry = targetParent->GetChild(to.filename().string());
    if (preexistingTargetEntry != boost::none && preexistingTargetEntry->type() == fspp::Dir::EntryType::DIR) {
      if (getType() != fspp::Dir::EntryType::DIR) {
        // A directory cannot be overwritten with a non-directory
        throw FuseErrnoException(EISDIR);
      }
      auto preexistingTarget = device()->LoadBlob(preexistingTargetEntry->blockId());
      auto preexistingTargetDir = dynamic_pointer_move<DirBlobRef>(preexistingTarget);
      if (preexistingTargetDir == none) {
        LOG(ERR, "Preexisting target is not a directory. But its parent dir entry says it's a directory");
        throw FuseErrnoException(EIO);
      }
      if ((*preexistingTargetDir)->NumChildren() > 0) {
        // Cannot overwrite a non-empty dir with a rename operation.
        throw FuseErrnoException(ENOTEMPTY);
      }
    }

    _updateParentModificationTimestamp();
    _updateTargetDirModificationTimestamp(*targetParent, std::move(targetGrandparent));
    targetParent->AddOrOverwriteChild(to.filename().string(), oldEntry.blockId(), oldEntry.type(), oldEntry.mode(), oldEntry.uid(), oldEntry.gid(),
                                  oldEntry.lastAccessTime(), oldEntry.lastModificationTime(), onOverwritten);
    (*_parent)->RemoveChild(oldEntry.name());
    // targetParent is now the new parent for this node. Adapt to it, so we can call further operations on this node object.
    LoadBlob()->setParentPointer(targetParent->blockId());
    _parent = std::move(targetParent);
  }
}

void CryNode::_updateParentModificationTimestamp() {
  if (_grandparent != none) {
    // TODO Handle timestamps of the root directory (_grandparent == none) correctly.
    ASSERT(_parent != none, "Grandparent is set, so also parent has to be set");
    (*_grandparent)->updateModificationTimestampForChild((*_parent)->blockId());
  }
}

void CryNode::_updateTargetDirModificationTimestamp(const DirBlobRef &targetDir, optional<unique_ref<DirBlobRef>> targetDirParent) {
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

unique_ref<FsBlobRef> CryNode::LoadBlob() const {
  auto blob = _device->LoadBlob(_blockId);
  ASSERT(_parent == none || blob->parentPointer() == (*_parent)->blockId(), "Blob has wrong parent pointer.");
  return blob;  // NOLINT (workaround https://gcc.gnu.org/bugzilla/show_bug.cgi?id=82481 )
}

const blockstore::BlockId &CryNode::blockId() const {
  return _blockId;
}

CryNode::stat_info CryNode::stat() const {
  device()->callFsActionCallbacks();
  if(_parent == none) {
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
    const struct timespec now = cpputils::time::now();
    result.atime = now;
    result.mtime = now;
    result.ctime = now;
    return result;
  } else {
    auto childOpt = (*_parent)->GetChild(_blockId);
    if (childOpt == boost::none) {
      throw fspp::fuse::FuseErrnoException(ENOENT);
    }
    return dirEntryToStatInfo(*childOpt, LoadBlob()->lstat_size());
  }
}

void CryNode::chmod(fspp::mode_t mode) {
  device()->callFsActionCallbacks();
  if (_parent == none) {
    //We are the root direcory.
	//TODO What should we do?
	return;
  }
  (*_parent)->chmodChild(_blockId, mode);
}

void CryNode::chown(fspp::uid_t uid, fspp::gid_t gid) {
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
