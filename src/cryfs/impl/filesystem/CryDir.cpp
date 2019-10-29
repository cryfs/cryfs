#include "CryDir.h"

#include <fspp/fs_interface/FuseErrnoException.h>
#include "CryDevice.h"
#include "CryFile.h"
#include "CryOpenFile.h"
#include <cpp-utils/system/time.h>

//TODO Get rid of this in favor of exception hierarchy
using fspp::fuse::FuseErrnoException;

namespace bf = boost::filesystem;

using std::string;
using std::vector;

using blockstore::BlockId;
using cpputils::unique_ref;
using cpputils::make_unique_ref;
using cpputils::dynamic_pointer_move;
using boost::none;
using cryfs::parallelaccessfsblobstore::DirBlobRef;

namespace cryfs {

CryDir::CryDir(CryDevice *device, const BlockId &blockId)
: CryNode(device, blockId) {
}

CryDir::~CryDir() = default;

unique_ref<fspp::OpenFile> CryDir::createAndOpenFile(const string &name, fspp::mode_t mode, fspp::uid_t uid, fspp::gid_t gid) {
  device()->callFsActionCallbacks();
  auto blob = LoadDirBlob();
  auto now = cpputils::time::now();
  FsBlobView::Metadata metaData(uint32_t{1}, mode, uid, gid, fspp::num_bytes_t{0}, now, now, now);
  auto child = device()->CreateFileBlob(metaData);
  blob->AddChildFile(name, child->blockId());
  blob->link();
  return make_unique_ref<CryOpenFile>(device(), std::move(child));
}

void CryDir::createDir(const string &name, fspp::mode_t mode, fspp::uid_t uid, fspp::gid_t gid) {
  device()->callFsActionCallbacks();
  auto blob = LoadDirBlob();
  auto now = cpputils::time::now();
  FsBlobView::Metadata metaData(uint32_t{2}, mode, uid, gid, fspp::num_bytes_t{0}, now, now, now);
  auto child = device()->CreateDirBlob(metaData);
  blob->AddChildDir(name, child->blockId());
  blob->link();
}

unique_ref<DirBlobRef> CryDir::LoadDirBlob() const {
  auto blob = CryNode::LoadBlob();
  auto dir_blob = dynamic_pointer_move<DirBlobRef>(blob);
  ASSERT(dir_blob != none, "Blob does not store a directory");
  return std::move(*dir_blob);
}

vector<fspp::Dir::Entry> CryDir::children() {
  device()->callFsActionCallbacks();
  updateAccessTimestamp();
  vector<fspp::Dir::Entry> children;
  children.emplace_back(fspp::Dir::EntryType::DIR, ".");
  children.emplace_back(fspp::Dir::EntryType::DIR, "..");
  auto blob = LoadDirBlob();
  blob->AppendChildrenTo(&children);
  return children;
}

fspp::Dir::EntryType CryDir::getType() const {
  device()->callFsActionCallbacks();
  return fspp::Dir::EntryType::DIR;
}

void CryDir::createSymlink(const string &name, const bf::path &target, fspp::uid_t uid, fspp::gid_t gid) {
  device()->callFsActionCallbacks();
  auto blob = LoadDirBlob();
  auto now = cpputils::time::now();
  fspp::mode_t mode(0120777);
  FsBlobView::Metadata metaData(uint32_t {1}, mode, uid, gid, fspp::num_bytes_t{0}, now, now, now);
  auto child = device()->CreateSymlinkBlob(target, metaData);
  blob->AddChildSymlink(name, child->blockId());
  blob->link();
}

void CryDir::remove() {
  device()->callFsActionCallbacks();
  {
    auto blob = LoadDirBlob();
    if (0 != blob->NumChildren()) {
      throw FuseErrnoException(ENOTEMPTY);
    }
  }
  //TODO removeNode() calls CryDevice::RemoveBlob, which loads the blob again. So we're loading it twice. Should be optimized.
  removeNode();
}

void CryDir::updateAccessTimestamp() {
  auto blob = LoadDirBlob();
  (*blob).updateAccessTimestamp();
}

void CryDir::updateModificationTimestamp() {
  auto blob = LoadDirBlob();
  (*blob).updateModificationTimestamp();
}

void CryDir::updateChangeTimestamp() {
  auto blob = LoadDirBlob();
  (*blob).updateChangeTimestamp();
}

void CryDir::removeChildEntryByName(const string &name) {
  auto blob = LoadDirBlob();
  blob->updateChangeTimestamp();
  blob->updateModificationTimestamp();
  blob->unlink();
  LoadDirBlob() ->RemoveChild(name);
}

void CryDir::createLink(const boost::filesystem::path &target, const std::string& name) {
  device()->callFsActionCallbacks();

  // TODO: before, or after, or only on reset?
  updateChangeTimestamp();
  updateModificationTimestamp();

  // TODO(joka921) Implement LoadAndLink to save blobs from deletion while we are doing something
  // with them?
  auto targetBlob = device()->Load(target);
  if (targetBlob == none) {
    throw FuseErrnoException(ENOENT);
  }
  auto type = (*targetBlob)->getType();
  if (type == fspp::Dir::EntryType::DIR) {
    throw FuseErrnoException(EPERM);
  }
  (*targetBlob)->link(); // now we are save from deletion

  try {
    LoadDirBlob()->AddChildHardlink(name, (*targetBlob)->blockId(), (*targetBlob)->getType());
  } catch (const FuseErrnoException& e) {
    (*targetBlob)->unlink();
    throw;
  }
}

}
