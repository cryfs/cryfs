#include "CryNode.h"

#include <sys/time.h>

#include "CryDevice.h"
#include "CryDir.h"
#include "CryFile.h"
#include "messmer/fspp/fuse/FuseErrnoException.h"
#include <messmer/cpp-utils/pointer.h>

namespace bf = boost::filesystem;

using std::unique_ptr;
using blockstore::Key;
using blobstore::Blob;
using cpputils::dynamic_pointer_move;

//TODO Get rid of this in favor of an exception hierarchy
using fspp::fuse::CHECK_RETVAL;
using fspp::fuse::FuseErrnoException;

namespace cryfs {

CryNode::CryNode(CryDevice *device, unique_ptr<DirBlob> parent, const Key &key)
: _device(device),
  _parent(std::move(parent)),
  _key(key) {
}

CryNode::~CryNode() {
}

void CryNode::access(int mask) const {
  return;
  throw FuseErrnoException(ENOTSUP);
}

void CryNode::rename(const bf::path &to) {
  //TODO More efficient implementation possible: directly rename when it's actually not moved to a different directory
  _parent->RemoveChild(_key);
  _parent->flush();
  auto targetDir = _device->LoadDirBlob(to.parent_path());
  targetDir->AddChild(to.filename().native(), _key, getType());
}

void CryNode::utimens(const timespec times[2]) {
  throw FuseErrnoException(ENOTSUP);
}

void CryNode::remove() {
  _parent->RemoveChild(_key);
  _device->RemoveBlob(_key);
}

CryDevice *CryNode::device() {
  return _device;
}

const CryDevice *CryNode::device() const {
  return _device;
}

unique_ptr<Blob> CryNode::LoadBlob() const {
  return _device->LoadBlob(_key);
}

}
