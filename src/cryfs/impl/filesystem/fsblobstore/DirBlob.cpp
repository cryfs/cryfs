#include "DirBlob.h"
#include <cassert>

//TODO Remove and replace with exception hierarchy
#include <fspp/fs_interface/FuseErrnoException.h>

#include <blobstore/implementations/onblocks/utils/Math.h>
#include <cpp-utils/data/Data.h>
#include "cryfs/impl/filesystem/CryDevice.h"
#include "FileBlob.h"
#include "SymlinkBlob.h"
#include <cpp-utils/system/stat.h>

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

constexpr fspp::num_bytes_t DirBlob::DIR_LSTAT_SIZE;

DirBlob::DirBlob(unique_ref<Blob> blob, std::function<fspp::num_bytes_t (const blockstore::BlockId&)> getLstatSize) :
    FsBlob(std::move(blob)), _getLstatSize(getLstatSize), _getLstatSizeMutex(), _entries(), _entriesAndChangedMutex(), _changed(false) {
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

unique_ref<DirBlob> DirBlob::InitializeEmptyDir(unique_ref<Blob> blob, const blockstore::BlockId &parent, std::function<fspp::num_bytes_t(const blockstore::BlockId&)> getLstatSize) {
  InitializeBlob(blob.get(), FsBlobView::BlobType::DIR, parent);
  return make_unique_ref<DirBlob>(std::move(blob), getLstatSize);
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
  std::unique_lock<std::mutex> lock(_entriesAndChangedMutex);
  _addChild(name, blobId, fspp::Dir::EntryType::DIR, mode, uid, gid, lastAccessTime, lastModificationTime);
}

void DirBlob::AddChildFile(const std::string &name, const BlockId &blobId, fspp::mode_t mode, fspp::uid_t uid, fspp::gid_t gid, timespec lastAccessTime, timespec lastModificationTime) {
  std::unique_lock<std::mutex> lock(_entriesAndChangedMutex);
  _addChild(name, blobId, fspp::Dir::EntryType::FILE, mode, uid, gid, lastAccessTime, lastModificationTime);
}

void DirBlob::AddChildSymlink(const std::string &name, const blockstore::BlockId &blobId, fspp::uid_t uid, fspp::gid_t gid, timespec lastAccessTime, timespec lastModificationTime) {
  std::unique_lock<std::mutex> lock(_entriesAndChangedMutex);
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
  std::unique_lock<std::mutex> lock(_entriesAndChangedMutex);
  _entries.addOrOverwrite(name, blobId, entryType, mode, uid, gid, lastAccessTime, lastModificationTime, onOverwritten);
  _changed = true;
}

void DirBlob::RenameChild(const blockstore::BlockId &blockId, const std::string &newName, std::function<void (const blockstore::BlockId &blockId)> onOverwritten) {
  std::unique_lock<std::mutex> lock(_entriesAndChangedMutex);
  _entries.rename(blockId, newName, onOverwritten);
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
  _changed = true;
}

void DirBlob::AppendChildrenTo(vector<fspp::Dir::Entry> *result) const {
  std::unique_lock<std::mutex> lock(_entriesAndChangedMutex);
  result->reserve(result->size() + _entries.size());
  for (const auto &entry : _entries) {
    result->emplace_back(entry.type(), entry.name());
  }
}

fspp::num_bytes_t DirBlob::lstat_size() const {
  return DIR_LSTAT_SIZE;
}

fspp::Node::stat_info DirBlob::statChild(const BlockId &blockId) const {
  std::unique_lock<std::mutex> lock(_getLstatSizeMutex);
  auto lstatSizeGetter = _getLstatSize;

  // The following unlock is important to avoid deadlock.
  // ParallelAccessFsBlobStore::load() causes a call to DirBlob::setLstatSizeGetter,
  // so their lock ordering first locks the ParallelAccessStore::_mutex, then the DirBlob::_getLstatSizeMutex.
  // this requires us to free DirBlob::_getLstatSizeMutex before calling into lstatSizeGetter(), because
  // lstatSizeGetter can call ParallelAccessFsBlobStore::load().
  lock.unlock();

  auto lstatSize = lstatSizeGetter(blockId);
  return statChildWithKnownSize(blockId, lstatSize);
}

fspp::Node::stat_info DirBlob::statChildWithKnownSize(const BlockId &blockId, fspp::num_bytes_t size) const {
  fspp::Node::stat_info result;

  auto childOpt = GetChild(blockId);
  if (childOpt == boost::none) {
    throw fspp::fuse::FuseErrnoException(ENOENT);
  }
  const auto &child = *childOpt;
  result.mode = child.mode();
  result.uid = child.uid();
  result.gid = child.gid();
  //TODO If possible without performance loss, then for a directory, st_nlink should return number of dir entries (including "." and "..")
  result.nlink = 1;
  result.size = size;
  result.atime = child.lastAccessTime();
  result.mtime = child.lastModificationTime();
  result.ctime = child.lastMetadataChangeTime();
  //TODO Move ceilDivision to general utils which can be used by cryfs as well
  result.blocks = blobstore::onblocks::utils::ceilDivision(size.value(), static_cast<int64_t>(512));
  return result;
}

void DirBlob::updateAccessTimestampForChild(const BlockId &blockId, fspp::TimestampUpdateBehavior timestampUpdateBehavior) {
  std::unique_lock<std::mutex> lock(_entriesAndChangedMutex);
  if (_entries.updateAccessTimestampForChild(blockId, timestampUpdateBehavior)) {
    _changed = true;
  }
}

void DirBlob::updateModificationTimestampForChild(const BlockId &blockId) {
  std::unique_lock<std::mutex> lock(_entriesAndChangedMutex);
  _entries.updateModificationTimestampForChild(blockId);
  _changed = true;
}

void DirBlob::chmodChild(const BlockId &blockId, fspp::mode_t mode) {
  std::unique_lock<std::mutex> lock(_entriesAndChangedMutex);
  _entries.setMode(blockId, mode);
  _changed = true;
}

void DirBlob::chownChild(const BlockId &blockId, fspp::uid_t uid, fspp::gid_t gid) {
  std::unique_lock<std::mutex> lock(_entriesAndChangedMutex);
  if(_entries.setUidGid(blockId, uid, gid)) {
    _changed = true;
  }
}

void DirBlob::utimensChild(const BlockId &blockId, timespec lastAccessTime, timespec lastModificationTime) {
  std::unique_lock<std::mutex> lock(_entriesAndChangedMutex);
  _entries.setAccessTimes(blockId, lastAccessTime, lastModificationTime);
  _changed = true;
}

void DirBlob::setLstatSizeGetter(std::function<fspp::num_bytes_t(const blockstore::BlockId&)> getLstatSize) {
    std::lock_guard<std::mutex> lock(_getLstatSizeMutex);
    _getLstatSize = std::move(getLstatSize);
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
