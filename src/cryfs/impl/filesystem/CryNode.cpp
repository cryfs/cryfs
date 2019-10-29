#include "CryNode.h"

#include "CryDevice.h"
#include "CryFile.h"
#include <fspp/fs_interface/FuseErrnoException.h>
#include <cpp-utils/system/time.h>

namespace bf = boost::filesystem;

using blockstore::BlockId;
using cpputils::unique_ref;
using boost::none;
using cryfs::parallelaccessfsblobstore::FsBlobRef;
using namespace cpputils::logging;

//TODO Get rid of this in favor of an exception hierarchy
using fspp::fuse::FuseErrnoException;

namespace cryfs {

CryNode::CryNode(CryDevice *device, const BlockId &blockId)
        : _device(device),
          _blockId(blockId) {
}

CryNode::~CryNode() = default;

void CryNode::access(int mask) const {
  // TODO Should we implement access()?
  UNUSED(mask);
  device()->callFsActionCallbacks();
}


void CryNode::rename(const bf::path &from, const bf::path &to) {
  device()->callFsActionCallbacks();


  if (isRootDir()) {
    //We are the root direcory.
    throw FuseErrnoException(EBUSY);
  }

  if (from == to) {
    updateChangeTimestamp();
    return; // rename to self doesn't do anything
  }

  auto myBlob = LoadBlob();
  auto targetDir = _device->LoadDirBlob(to.parent_path());

  auto oldParent = _device->LoadDirBlob(from.parent_path());
  auto old = oldParent->GetChild(_blockId);
  if (old == boost::none) {
    throw FuseErrnoException(EIO);
  }
  fsblobstore::DirEntry oldEntry = *old; // Copying this (instead of only keeping the reference) is necessary, because the operations below (i.e. RenameChild()) might make a reference invalid.

  auto pair = std::mismatch(from.begin(), from.end(), to.begin(), to.end());
  if (pair.first == from.end()) {
    // renaming directory to child of itself -> illegal
    throw FuseErrnoException(EINVAL);
  }

  class HardlinkToSameException : public std::exception {};

  bool isDir = getType() == fspp::Dir::EntryType::DIR;

  // basically reimplement unlink() here.
  auto onOverwritten = [this, isDir](const fsblobstore::DirEntry &entry) {
    if (_blockId == entry.blockId()) {
      throw HardlinkToSameException();
    }

    // trying to overwrite a directory
    if (entry.type() == fspp::Dir::EntryType::DIR) {
      if (!isDir) throw FuseErrnoException(EISDIR);
      // only allowed if target is empty
      auto remove = device()->LoadDirBlob(entry.blockId())->NumChildren() == 0;
      if (remove) {
        device()->RemoveBlob(entry.blockId());
      } else {
        throw FuseErrnoException(ENOTEMPTY);
      }
    } else { // overwriting a regular file
      bool remove = device()->LoadBlob(entry.blockId())->unlink(); // immediately release the Blob
      if (remove) {
        device()->RemoveBlob(entry.blockId());
      }
    }
  };

  try {
    if (targetDir->blockId() == oldParent->blockId()) {
      targetDir->RenameChild(oldEntry.blockId(), to.filename().string(), onOverwritten);
    } else {
      targetDir->AddOrOverwriteChild(to.filename().string(), oldEntry.blockId(), oldEntry.type(),
                                     std::move(onOverwritten));
      oldParent->RemoveChild(blockId());
    }
  } catch (const HardlinkToSameException& h) {
    updateChangeTimestamp();
    return;
  } catch (const std::exception& e) {
    throw;
  }
  updateChangeTimestamp();

}

void CryNode::utimens(timespec lastAccessTime, timespec lastModificationTime) {
  LoadBlob()->utimens(lastAccessTime, lastModificationTime);
}

void CryNode::removeNode() {
  if (isRootDir()) {
    //We are the root direcory.
    //TODO What should we do?
    throw FuseErrnoException(EIO);
  }
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
  return blob;  // NOLINT (workaround https://gcc.gnu.org/bugzilla/show_bug.cgi?id=82481 )
}

const blockstore::BlockId &CryNode::blockId() const {
  return _blockId;
}

CryNode::stat_info CryNode::stat() const {
  device()->callFsActionCallbacks();
  return LoadBlob()->stat();
}

void CryNode::chmod(fspp::mode_t mode) {
  device()->callFsActionCallbacks();
  LoadBlob()->chmod(mode);
}

void CryNode::chown(fspp::uid_t uid, fspp::gid_t gid) {
  device()->callFsActionCallbacks();
  LoadBlob()->chown(uid, gid);
}

void CryNode::updateChangeTimestamp() {
  auto blob = LoadBlob();
  (*blob).updateChangeTimestamp();
}

bool CryNode::isRootDir() const {
  return (_blockId == device()->rootBlobId());
}

void CryNode::link() {
  LoadBlob()->link();
}

bool CryNode::unlink() {
  return LoadBlob()->unlink();
}

}
