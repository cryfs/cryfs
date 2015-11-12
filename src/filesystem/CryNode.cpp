#include "CryNode.h"

#include <sys/time.h>

#include "CryDevice.h"
#include "CryDir.h"
#include "CryFile.h"
#include "messmer/fspp/fuse/FuseErrnoException.h"
#include <messmer/cpp-utils/pointer/cast.h>

namespace bf = boost::filesystem;

using blockstore::Key;
using blobstore::Blob;
using cpputils::dynamic_pointer_move;
using cpputils::unique_ref;
using boost::optional;
using boost::none;
using cryfs::parallelaccessfsblobstore::FsBlobRef;
using cryfs::parallelaccessfsblobstore::DirBlobRef;

//TODO Get rid of this in favor of an exception hierarchy
using fspp::fuse::CHECK_RETVAL;
using fspp::fuse::FuseErrnoException;

namespace cryfs {

CryNode::CryNode(CryDevice *device, optional<unique_ref<DirBlobRef>> parent, const Key &key)
: _device(device),
  _parent(std::move(parent)),
  _key(key) {
}

CryNode::~CryNode() {
}

void CryNode::access(int mask) const {
  device()->callFsActionCallbacks();
  //TODO
  return;
  throw FuseErrnoException(ENOTSUP);
}

void CryNode::rename(const bf::path &to) {
  device()->callFsActionCallbacks();
  if (_parent == none) {
    //We are the root direcory.
    //TODO What should we do?
    throw FuseErrnoException(EIO);
  }
  //TODO More efficient implementation possible: directly rename when it's actually not moved to a different directory
  //     It's also quite ugly code because in the parent==targetDir case, it depends on _parent not overriding the changes made by targetDir.
  auto old = (*_parent)->GetChild(_key);
  auto mode = old.mode;
  auto uid = old.uid;
  auto gid = old.gid;
  (*_parent)->RemoveChild(_key);
  (*_parent)->flush();
  auto targetDir = _device->LoadDirBlob(to.parent_path());
  targetDir->AddChild(to.filename().native(), _key, getType(), mode, uid, gid);
}

void CryNode::utimens(const timespec times[2]) {
  device()->callFsActionCallbacks();
  //TODO
  throw FuseErrnoException(ENOTSUP);
}

void CryNode::remove() {
  device()->callFsActionCallbacks();
  //TODO Instead of all these if-else and having _parent being an optional, we could also introduce a CryRootDir which inherits from fspp::Dir.
  if (_parent == none) {
    //We are the root direcory.
    //TODO What should we do?
    throw FuseErrnoException(EIO);
  }
  (*_parent)->RemoveChild(_key);
  _device->RemoveBlob(_key);
}

CryDevice *CryNode::device() {
  return _device;
}

const CryDevice *CryNode::device() const {
  return _device;
}

unique_ref<FsBlobRef> CryNode::LoadBlob() const {
  return _device->LoadBlob(_key);
}

void CryNode::stat(struct ::stat *result) const {
  device()->callFsActionCallbacks();
  if(_parent == none) {
    //We are the root directory.
	//TODO What should we do?
	result->st_mode = S_IFDIR;
  } else {
    (*_parent)->statChild(_key, result);
  }
}

void CryNode::chmod(mode_t mode) {
  device()->callFsActionCallbacks();
  if (_parent == none) {
    //We are the root direcory.
	//TODO What should we do?
	throw FuseErrnoException(EIO);
  }
  (*_parent)->chmodChild(_key, mode);
}

void CryNode::chown(uid_t uid, gid_t gid) {
  device()->callFsActionCallbacks();
  if (_parent == none) {
	//We are the root direcory.
	//TODO What should we do?
	throw FuseErrnoException(EIO);
  }
  (*_parent)->chownChild(_key, uid, gid);
}

}
