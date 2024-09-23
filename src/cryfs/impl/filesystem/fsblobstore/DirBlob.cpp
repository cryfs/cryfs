#include "DirBlob.h"
#include <boost/optional/detail/optional_reference_spec.hpp>

//TODO Remove and replace with exception hierarchy
#include <cstddef>
#include <cstdint>
#include <ctime>

#include "blobstore/interface/Blob.h"
#include "blockstore/utils/BlockId.h"
#include "cpp-utils/assert/assert.h"
#include "cpp-utils/pointer/unique_ref.h"
#include "cryfs/impl/filesystem/fsblobstore/FsBlob.h"
#include "cryfs/impl/filesystem/fsblobstore/FsBlobView.h"
#include "cryfs/impl/filesystem/fsblobstore/utils/DirEntry.h"
#include "fspp/fs_interface/Context.h"
#include "fspp/fs_interface/Dir.h"
#include "fspp/fs_interface/Types.h"
#include <cpp-utils/data/Data.h>
#include <functional>
#include <mutex>
#include <string>
#include <utility>

using std::vector;
using std::string;

using blobstore::Blob;
using blockstore::BlockId;
using cpputils::Data;
using cpputils::unique_ref;
using cpputils::make_unique_ref;

namespace cryfs {
namespace fsblobstore {

constexpr fspp::num_bytes_t DirBlob::DIR_LSTAT_SIZE;

DirBlob::DirBlob(unique_ref<Blob> blob) :
    FsBlob(std::move(blob)), _entries(), _entriesAndChangedMutex(), _changed(false) {
  ASSERT(baseBlob().blobType() == FsBlobView::BlobType::DIR, "Loaded blob is not a directory");
  _readEntriesFromBlob();
}

DirBlob::~DirBlob() {
  const std::unique_lock<std::mutex> lock(_entriesAndChangedMutex);
  _writeEntriesToBlob();
}

void DirBlob::flush() {
  const std::unique_lock<std::mutex> lock(_entriesAndChangedMutex);
  _writeEntriesToBlob();
  baseBlob().flush();
}

unique_ref<DirBlob> DirBlob::InitializeEmptyDir(unique_ref<Blob> blob, const blockstore::BlockId &parent) {
  InitializeBlob(blob.get(), FsBlobView::BlobType::DIR, parent);
  return make_unique_ref<DirBlob>(std::move(blob));
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

void DirBlob::AddChildDir(const std::string &name, const BlockId &blobId, fspp::mode_t mode, fspp::uid_t uid, fspp::gid_t gid, timespec lastAccessTime, timespec lastModificationTime) {
  const std::unique_lock<std::mutex> lock(_entriesAndChangedMutex);
  _addChild(name, blobId, fspp::Dir::EntryType::DIR, mode, uid, gid, lastAccessTime, lastModificationTime);
}

void DirBlob::AddChildFile(const std::string &name, const BlockId &blobId, fspp::mode_t mode, fspp::uid_t uid, fspp::gid_t gid, timespec lastAccessTime, timespec lastModificationTime) {
  const std::unique_lock<std::mutex> lock(_entriesAndChangedMutex);
  _addChild(name, blobId, fspp::Dir::EntryType::FILE, mode, uid, gid, lastAccessTime, lastModificationTime);
}

void DirBlob::AddChildSymlink(const std::string &name, const blockstore::BlockId &blobId, fspp::uid_t uid, fspp::gid_t gid, timespec lastAccessTime, timespec lastModificationTime) {
  const std::unique_lock<std::mutex> lock(_entriesAndChangedMutex);
  auto mode = fspp::mode_t().addSymlinkFlag()
          .addUserReadFlag().addUserWriteFlag().addUserExecFlag()
          .addGroupReadFlag().addGroupWriteFlag().addGroupExecFlag()
          .addOtherReadFlag().addOtherWriteFlag().addOtherExecFlag();
  _addChild(name, blobId, fspp::Dir::EntryType::SYMLINK, mode, uid, gid, lastAccessTime, lastModificationTime);
}

void DirBlob::_addChild(const std::string &name, const BlockId &blobId,
    fspp::Dir::EntryType entryType, fspp::mode_t mode, fspp::uid_t uid, fspp::gid_t gid, timespec lastAccessTime, timespec lastModificationTime) {
  _entries.add(name, blobId, entryType, mode, uid, gid, lastAccessTime, lastModificationTime);
  _changed = true;
}

void DirBlob::AddOrOverwriteChild(const std::string &name, const BlockId &blobId, fspp::Dir::EntryType entryType,
                                  fspp::mode_t mode, fspp::uid_t uid, fspp::gid_t gid, timespec lastAccessTime, timespec lastModificationTime,
                                  std::function<void (const blockstore::BlockId &blockId)> onOverwritten) {
  const std::unique_lock<std::mutex> lock(_entriesAndChangedMutex);
  _entries.addOrOverwrite(name, blobId, entryType, mode, uid, gid, lastAccessTime, lastModificationTime, onOverwritten);
  _changed = true;
}

void DirBlob::RenameChild(const blockstore::BlockId &blockId, const std::string &newName, std::function<void (const blockstore::BlockId &blockId)> onOverwritten) {
  const std::unique_lock<std::mutex> lock(_entriesAndChangedMutex);
  _entries.rename(blockId, newName, onOverwritten);
  _changed = true;
}

boost::optional<const DirEntry&> DirBlob::GetChild(const string &name) const {
  const std::unique_lock<std::mutex> lock(_entriesAndChangedMutex);
  return _entries.get(name);
}

boost::optional<const DirEntry&> DirBlob::GetChild(const BlockId &blockId) const {
  const std::unique_lock<std::mutex> lock(_entriesAndChangedMutex);
  return _entries.get(blockId);
}

void DirBlob::RemoveChild(const string &name) {
  const std::unique_lock<std::mutex> lock(_entriesAndChangedMutex);
  _entries.remove(name);
  _changed = true;
}

void DirBlob::RemoveChild(const BlockId &blockId) {
  const std::unique_lock<std::mutex> lock(_entriesAndChangedMutex);
  _entries.remove(blockId);
  _changed = true;
}

void DirBlob::AppendChildrenTo(vector<fspp::Dir::Entry> *result) const {
  const std::unique_lock<std::mutex> lock(_entriesAndChangedMutex);
  result->reserve(result->size() + _entries.size());
  for (const auto &entry : _entries) {
    result->emplace_back(entry.type(), entry.name());
  }
}

fspp::num_bytes_t DirBlob::lstat_size() const {
  return DIR_LSTAT_SIZE;
}

void DirBlob::updateAccessTimestampForChild(const BlockId &blockId, fspp::TimestampUpdateBehavior timestampUpdateBehavior) {
  const std::unique_lock<std::mutex> lock(_entriesAndChangedMutex);
  if (_entries.updateAccessTimestampForChild(blockId, timestampUpdateBehavior)) {
    _changed = true;
  }
}

void DirBlob::updateModificationTimestampForChild(const BlockId &blockId) {
  const std::unique_lock<std::mutex> lock(_entriesAndChangedMutex);
  _entries.updateModificationTimestampForChild(blockId);
  _changed = true;
}

void DirBlob::chmodChild(const BlockId &blockId, fspp::mode_t mode) {
  const std::unique_lock<std::mutex> lock(_entriesAndChangedMutex);
  _entries.setMode(blockId, mode);
  _changed = true;
}

void DirBlob::chownChild(const BlockId &blockId, fspp::uid_t uid, fspp::gid_t gid) {
  const std::unique_lock<std::mutex> lock(_entriesAndChangedMutex);
  if(_entries.setUidGid(blockId, uid, gid)) {
    _changed = true;
  }
}

void DirBlob::utimensChild(const BlockId &blockId, timespec lastAccessTime, timespec lastModificationTime) {
  const std::unique_lock<std::mutex> lock(_entriesAndChangedMutex);
  _entries.setAccessTimes(blockId, lastAccessTime, lastModificationTime);
  _changed = true;
}

cpputils::unique_ref<blobstore::Blob> DirBlob::releaseBaseBlob() {
  const std::unique_lock<std::mutex> lock(_entriesAndChangedMutex);
  _writeEntriesToBlob();
  return FsBlob::releaseBaseBlob();
}

size_t DirBlob::NumChildren() const {
  return _entries.size();
}

}
}
