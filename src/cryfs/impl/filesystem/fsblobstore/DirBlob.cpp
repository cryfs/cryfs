#include "DirBlob.h"
#include <cassert>

//TODO Remove and replace with exception hierarchy
#include <fspp/fuse/FuseErrnoException.h>

#include <blobstore/implementations/onblocks/utils/Math.h>
#include <cpp-utils/data/Data.h>
#include "../CryDevice.h"
#include "FileBlob.h"
#include "SymlinkBlob.h"
#include <cpp-utils/system/stat.h>

using std::vector;
using std::string;
using std::pair;
using std::make_pair;

using blobstore::Blob;
using blockstore::Key;
using cpputils::Data;
using cpputils::unique_ref;
using cpputils::make_unique_ref;
using boost::none;

namespace cryfs {
namespace fsblobstore {

constexpr off_t DirBlob::DIR_LSTAT_SIZE;

DirBlob::DirBlob(FsBlobStore *fsBlobStore, unique_ref<Blob> blob, std::function<off_t (const blockstore::Key&)> getLstatSize) :
    FsBlob(std::move(blob)), _fsBlobStore(fsBlobStore), _getLstatSize(getLstatSize), _entries(), _mutex(), _changed(false) {
  ASSERT(baseBlob().blobType() == FsBlobView::BlobType::DIR, "Loaded blob is not a directory");
  _readEntriesFromBlob();
}

DirBlob::~DirBlob() {
  std::unique_lock<std::mutex> lock(_mutex);
  _writeEntriesToBlob();
}

void DirBlob::flush() {
  std::unique_lock<std::mutex> lock(_mutex);
  _writeEntriesToBlob();
  baseBlob().flush();
}

unique_ref<DirBlob> DirBlob::InitializeEmptyDir(FsBlobStore *fsBlobStore, unique_ref<Blob> blob, std::function<off_t(const blockstore::Key&)> getLstatSize) {
  InitializeBlob(blob.get(), FsBlobView::BlobType::DIR);
  return make_unique_ref<DirBlob>(fsBlobStore, std::move(blob), getLstatSize);
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

void DirBlob::AddChildDir(const std::string &name, const Key &blobKey, mode_t mode, uid_t uid, gid_t gid, timespec lastAccessTime, timespec lastModificationTime) {
  std::unique_lock<std::mutex> lock(_mutex);
  _addChild(name, blobKey, fspp::Dir::EntryType::DIR, mode, uid, gid, lastAccessTime, lastModificationTime);
}

void DirBlob::AddChildFile(const std::string &name, const Key &blobKey, mode_t mode, uid_t uid, gid_t gid, timespec lastAccessTime, timespec lastModificationTime) {
  std::unique_lock<std::mutex> lock(_mutex);
  _addChild(name, blobKey, fspp::Dir::EntryType::FILE, mode, uid, gid, lastAccessTime, lastModificationTime);
}

void DirBlob::AddChildSymlink(const std::string &name, const blockstore::Key &blobKey, uid_t uid, gid_t gid, timespec lastAccessTime, timespec lastModificationTime) {
  std::unique_lock<std::mutex> lock(_mutex);
  _addChild(name, blobKey, fspp::Dir::EntryType::SYMLINK, S_IFLNK | S_IRUSR | S_IWUSR | S_IXUSR | S_IRGRP | S_IWGRP | S_IXGRP | S_IROTH | S_IWOTH | S_IXOTH, uid, gid, lastAccessTime, lastModificationTime);
}

void DirBlob::_addChild(const std::string &name, const Key &blobKey,
    fspp::Dir::EntryType entryType, mode_t mode, uid_t uid, gid_t gid, timespec lastAccessTime, timespec lastModificationTime) {
  _entries.add(name, blobKey, entryType, mode, uid, gid, lastAccessTime, lastModificationTime);
  _changed = true;
}

void DirBlob::AddOrOverwriteChild(const std::string &name, const Key &blobKey, fspp::Dir::EntryType entryType,
                                  mode_t mode, uid_t uid, gid_t gid, timespec lastAccessTime, timespec lastModificationTime,
                                  std::function<void (const blockstore::Key &key)> onOverwritten) {
  std::unique_lock<std::mutex> lock(_mutex);
  _entries.addOrOverwrite(name, blobKey, entryType, mode, uid, gid, lastAccessTime, lastModificationTime, onOverwritten);
  _changed = true;
}

void DirBlob::RenameChild(const blockstore::Key &key, const std::string &newName, std::function<void (const blockstore::Key &key)> onOverwritten) {
  std::unique_lock<std::mutex> lock(_mutex);
  _entries.rename(key, newName, onOverwritten);
  _changed = true;
}

boost::optional<const DirEntry&> DirBlob::GetChild(const string &name) const {
  std::unique_lock<std::mutex> lock(_mutex);
  return _entries.get(name);
}

boost::optional<const DirEntry&> DirBlob::GetChild(const Key &key) const {
  std::unique_lock<std::mutex> lock(_mutex);
  return _entries.get(key);
}

void DirBlob::RemoveChild(const string &name) {
  std::unique_lock<std::mutex> lock(_mutex);
  _entries.remove(name);
  _changed = true;
}

void DirBlob::RemoveChild(const Key &key) {
  std::unique_lock<std::mutex> lock(_mutex);
  _entries.remove(key);
  _changed = true;
}

void DirBlob::AppendChildrenTo(vector<fspp::Dir::Entry> *result) const {
  std::unique_lock<std::mutex> lock(_mutex);
  result->reserve(result->size() + _entries.size());
  for (const auto &entry : _entries) {
    result->emplace_back(entry.type(), entry.name());
  }
}

off_t DirBlob::lstat_size() const {
  return DIR_LSTAT_SIZE;
}

void DirBlob::statChild(const Key &key, struct ::stat *result) const {
  statChildExceptSize(key, result);
  result->st_size = _getLstatSize(key);
}

void DirBlob::statChildExceptSize(const Key &key, struct ::stat *result) const {
  auto childOpt = GetChild(key);
  if (childOpt == boost::none) {
    throw fspp::fuse::FuseErrnoException(ENOENT);
  }
  const auto &child = *childOpt;
  result->st_mode = child.mode();
  result->st_uid = child.uid();
  result->st_gid = child.gid();
  //TODO If possible without performance loss, then for a directory, st_nlink should return number of dir entries (including "." and "..")
  result->st_nlink = 1;
  result->st_atim = child.lastAccessTime();
  result->st_mtim = child.lastModificationTime();
  result->st_ctim = child.lastMetadataChangeTime();
  //TODO Move ceilDivision to general utils which can be used by cryfs as well
  result->st_blocks = blobstore::onblocks::utils::ceilDivision(result->st_size, (off_t)512);
  result->st_blksize = _fsBlobStore->virtualBlocksizeBytes();
}

void DirBlob::updateAccessTimestampForChild(const Key &key) {
  std::unique_lock<std::mutex> lock(_mutex);
  _entries.updateAccessTimestampForChild(key);
  _changed = true;
}

void DirBlob::updateModificationTimestampForChild(const Key &key) {
  std::unique_lock<std::mutex> lock(_mutex);
  _entries.updateModificationTimestampForChild(key);
  _changed = true;
}

void DirBlob::chmodChild(const Key &key, mode_t mode) {
  std::unique_lock<std::mutex> lock(_mutex);
  _entries.setMode(key, mode);
  _changed = true;
}

void DirBlob::chownChild(const Key &key, uid_t uid, gid_t gid) {
  std::unique_lock<std::mutex> lock(_mutex);
  if(_entries.setUidGid(key, uid, gid)) {
    _changed = true;
  }
}

void DirBlob::utimensChild(const Key &key, timespec lastAccessTime, timespec lastModificationTime) {
  std::unique_lock<std::mutex> lock(_mutex);
  _entries.setAccessTimes(key, lastAccessTime, lastModificationTime);
  _changed = true;
}

void DirBlob::setLstatSizeGetter(std::function<off_t(const blockstore::Key&)> getLstatSize) {
    std::unique_lock<std::mutex> lock(_mutex);
    _getLstatSize = getLstatSize;
}

cpputils::unique_ref<blobstore::Blob> DirBlob::releaseBaseBlob() {
  std::unique_lock<std::mutex> lock(_mutex);
  _writeEntriesToBlob();
  return FsBlob::releaseBaseBlob();
}

size_t DirBlob::NumChildren() const {
  return _entries.size();
}

}
}
