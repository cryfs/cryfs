#include "DirBlob.h"

//TODO Remove and replace with exception hierarchy
#include <fspp/fs_interface/FuseErrnoException.h>

#include <blobstore/implementations/onblocks/utils/Math.h>
#include <cpp-utils/data/Data.h>
#include "cryfs/impl/filesystem/CryDevice.h"
#include "FileBlob.h"
#include "SymlinkBlob.h"

using std::vector;
using std::string;

using blobstore::Blob;
using blockstore::BlockId;
using cpputils::Data;
using cpputils::unique_ref;
using cpputils::make_unique_ref;
using boost::none;

namespace cryfs {
namespace fsblobstore {

DirBlob::DirBlob(unique_ref<Blob> blob, const TimestampUpdateBehavior& behav) :
    FsBlob(std::move(blob), behav), _entries(), _entriesAndChangedMutex(), _changed(false) {
  ASSERT(baseBlob().blobType() == FsBlobView::BlobType::DIR, "Loaded blob is not a directory");
  _readEntriesFromBlob();
}

DirBlob::~DirBlob() {
  std::unique_lock<std::mutex> lock(_entriesAndChangedMutex);
  _writeEntriesToBlob();
}

void DirBlob::flush() {
  std::unique_lock<std::mutex> lock(_entriesAndChangedMutex);
  _writeEntriesToBlob();
  baseBlob().flush();
}

void DirBlob::utimens(timespec atime, timespec mtime) {
return baseBlob().utimens(atime, mtime);
}

unique_ref<DirBlob> DirBlob::InitializeEmptyDir(unique_ref<Blob> blob, const FsBlobView::Metadata &meta, const TimestampUpdateBehavior& updateBehavior) {
  InitializeBlob(blob.get(), meta, FsBlobView::BlobType::DIR);
  return make_unique_ref<DirBlob>(std::move(blob), updateBehavior);
}

void DirBlob::_writeEntriesToBlob() {
  if (_changed) {
    Data serialized = _entries.serialize();
    baseBlob().resize(serialized.size());
    baseBlob().write(serialized.data(), 0, serialized.size());
    _changed = false;
  }
}

void DirBlob::_readEntriesFromBlob() {
  //No lock needed, because this is only called from the constructor.
  Data data = baseBlob().readAll();
  _entries.deserializeFrom(static_cast<uint8_t*>(data.data()), data.size());
}

void DirBlob::AddChildDir(const std::string &name, const BlockId &blobId) {
  std::unique_lock<std::mutex> lock(_entriesAndChangedMutex);
  _addChild(name, blobId, fspp::Dir::NodeType::DIR);
}

void DirBlob::AddChildFile(const std::string &name, const BlockId &blobId) {
  std::unique_lock<std::mutex> lock(_entriesAndChangedMutex);
  _addChild(name, blobId, fspp::Dir::NodeType::FILE);
}

void DirBlob::AddChildSymlink(const std::string &name, const blockstore::BlockId &blobId) {
  std::unique_lock<std::mutex> lock(_entriesAndChangedMutex);
  _addChild(name, blobId, fspp::Dir::NodeType::SYMLINK);
}

void DirBlob::AddChildHardlink(const std::string& name, const blockstore::BlockId &blobId, const fspp::Dir::NodeType type) {
  std::unique_lock<std::mutex> lock(_entriesAndChangedMutex);
  auto existingChild = _entries.get(name);
  if (existingChild != none) {
    throw fspp::fuse::FuseErrnoException(EEXIST);
  }
  _addChild(name, blobId, type);
  link();
}

void DirBlob::_addChild(const std::string &name, const BlockId &blobId,
    fspp::Dir::NodeType entryType) {
  _entries.add(name, blobId, entryType);
  _changed = true;
}

void DirBlob::AddOrOverwriteChild(const std::string &name, const BlockId &blobId, fspp::Dir::NodeType entryType,
                                  const std::function<void (const DirEntry &entry)>& onOverwritten) {
  std::unique_lock<std::mutex> lock(_entriesAndChangedMutex);
  auto res = _entries.addOrOverwrite(name, blobId, entryType, onOverwritten);
  if (res == DirEntryList::AddOver::ADD) {
    link();
  }
  updateChangeTimestamp();
  updateModificationTimestamp();
  _changed = true;
}

void DirBlob::RenameChild(const blockstore::BlockId &blockId, const std::string &newName, const std::function<void (const DirEntry &entry)>& onOverwritten) {
  std::unique_lock<std::mutex> lock(_entriesAndChangedMutex);
  _entries.rename(blockId, newName, onOverwritten);
  updateModificationTimestamp();
  updateChangeTimestamp();

  _changed = true;
}

boost::optional<const DirEntry&> DirBlob::GetChild(const string &name) const {
  std::unique_lock<std::mutex> lock(_entriesAndChangedMutex);
  return _entries.get(name);
}

boost::optional<const DirEntry&> DirBlob::GetChild(const BlockId &blockId) const {
  std::unique_lock<std::mutex> lock(_entriesAndChangedMutex);
  return _entries.get(blockId);
}

void DirBlob::RemoveChild(const string &name) {
  std::unique_lock<std::mutex> lock(_entriesAndChangedMutex);
  _entries.remove(name);
  _changed = true;
}

void DirBlob::RemoveChild(const BlockId &blockId) {
  std::unique_lock<std::mutex> lock(_entriesAndChangedMutex);
  _entries.remove(blockId);
  unlink();
  updateModificationTimestamp();
  updateChangeTimestamp();
  _changed = true;
}

void DirBlob::AppendChildrenTo(vector<fspp::Dir::Entry> *result) const {
  std::unique_lock<std::mutex> lock(_entriesAndChangedMutex);
  result->reserve(result->size() + _entries.size());
  for (const auto &entry : _entries) {
    result->emplace_back(entry.type(), entry.name());
  }
}

cpputils::unique_ref<blobstore::Blob> DirBlob::releaseBaseBlob() {
  std::unique_lock<std::mutex> lock(_entriesAndChangedMutex);
  _writeEntriesToBlob();
  return FsBlob::releaseBaseBlob();
}

size_t DirBlob::NumChildren() const {
  return _entries.size();
}

}
}
