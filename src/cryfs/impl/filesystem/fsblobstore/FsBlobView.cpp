#include "FsBlobView.h"
#include "utils/DirEntry.h"
#include "utils/DirEntryList.h"

using cpputils::Data;

namespace cryfs {
constexpr uint16_t FsBlobView::FORMAT_VERSION_HEADER;
constexpr unsigned int FsBlobView::HEADER_SIZE;
constexpr fspp::num_bytes_t FsBlobView::DIR_LSTAT_SIZE;


#ifndef CRYFS_NO_COMPATIBILITY
std::vector<cryfs::fsblobstore::DirEntryWithMetaData> FsBlobView::migrate(blobstore::Blob *blob, Metadata metadata, BlobType type) {
  constexpr unsigned int VERY_OLD_HEADER_SIZE = sizeof(FORMAT_VERSION_HEADER) + sizeof(uint8_t);
  constexpr unsigned int OLD_HEADER_SIZE = sizeof(FORMAT_VERSION_HEADER) + sizeof(uint8_t) + blockstore::BlockId::BINARY_LENGTH;

  auto versionHeader = getFormatVersionHeader(*blob);
  unsigned int readHeaderSize;
  if (versionHeader == FORMAT_VERSION_HEADER) {
    // blob already migrated
    return {};
  } else if (versionHeader == 0) {
    readHeaderSize = VERY_OLD_HEADER_SIZE;
  } else if (versionHeader == 1) {
    readHeaderSize = OLD_HEADER_SIZE;
  } else {
    ASSERT(false, "Unknown format version header, are you using an older version of CryFS than what was used to setup this filesystem?");
  }

  fspp::mode_t realMode; // all bits 0 by default constructor
  if (type == BlobType::DIR) {
    realMode = realMode.addDirFlag();
  } else if (type == BlobType::SYMLINK) {
    realMode = realMode.addSymlinkFlag();
  } else {
    realMode = realMode.addFileFlag();
  }
  metadata._info.mode = realMode.changePermissions(metadata._info.mode);


  cpputils::Data data = blob->readAll();

  blob->resize(blob->size() + (HEADER_SIZE - readHeaderSize));
  blob->write(&FORMAT_VERSION_HEADER, 0, sizeof(FORMAT_VERSION_HEADER));
  blob->write(&metadata, sizeof(FORMAT_VERSION_HEADER), sizeof(metadata));
  static_assert(HEADER_SIZE == sizeof(FORMAT_VERSION_HEADER) + sizeof(metadata), "If this fails, the header is not initialized correctly in this function.");
  if (type == BlobType::SYMLINK || type == BlobType::FILE) {
    blob->write(data.dataOffset(readHeaderSize), HEADER_SIZE, data.size() - readHeaderSize);
    return {};
  } else  {
    // this is a directory blob which stores metadata about its descendants
    std::vector<cryfs::fsblobstore::DirEntryWithMetaData> entries;
    const char *pos = static_cast<const char*>(data.dataOffset(readHeaderSize));
    while (pos < static_cast<const char*>(data.data()) + data.size()) {
      cryfs::fsblobstore::DirEntryWithMetaData::deserializeAndAddToVector(pos, &entries);
      ASSERT(entries.size() == 1 || (entries[entries.size()-2]._blockId < entries[entries.size()-1]._blockId), "Invariant hurt: Directory entries should be ordered by blockId and unique in the old version format.");
    }
    std::vector<cryfs::fsblobstore::DirEntry> convertedEntries;
    for (const auto& e : entries) {
      convertedEntries.emplace_back(e._type, e._name, e._blockId);
    }

    cpputils::Data newData = cryfs::fsblobstore::DirEntryList::serializeExternal(convertedEntries);
    blob->resize(HEADER_SIZE + newData.size());
    blob->write(newData.data(), HEADER_SIZE, newData.size());
    return entries;


  }
}

#endif

void FsBlobView::InitializeBlob(blobstore::Blob *baseBlob, Metadata metadata, FsBlobView::BlobType type) {
  // manually set the type flags for safety and consistency. Only take permissions from metadatas mode
  fspp::mode_t realMode; // all bits 0 by default constructor
  if (type == BlobType::DIR) {
    realMode = realMode.addDirFlag();
  } else if (type == BlobType::SYMLINK) {
    realMode = realMode.addSymlinkFlag();
  } else {
    realMode = realMode.addFileFlag();
  }

  metadata._info.mode = realMode.changePermissions(metadata._info.mode);
  baseBlob->resize(HEADER_SIZE);
  baseBlob->write(&FORMAT_VERSION_HEADER, 0, sizeof(FORMAT_VERSION_HEADER));
  baseBlob->write(&metadata, sizeof(FORMAT_VERSION_HEADER), sizeof(metadata));
  static_assert(HEADER_SIZE == sizeof(FORMAT_VERSION_HEADER) + sizeof(metadata), "If this fails, the header is not initialized correctly in this function.");
}

void FsBlobView::resize(uint64_t numBytes) {
  Lock l(_mutex);
  _updateModificationTimestamp();
  _updateChangeTimestamp();
  _baseBlob->resize(numBytes + HEADER_SIZE);
  _metadata._info.size = fspp::num_bytes_t(_baseBlob->size() - HEADER_SIZE);
}

cpputils::Data FsBlobView::readAll() const {
  SharedLock l(_mutex);
  cpputils::Data data = _baseBlob->readAll();
  cpputils::Data dataWithoutHeader(data.size() - HEADER_SIZE);
  //Can we avoid this memcpy? Maybe by having Data::subdata() that returns a reference to the same memory region? Should we?
  std::memcpy(dataWithoutHeader.data(), data.dataOffset(HEADER_SIZE), dataWithoutHeader.size());
return dataWithoutHeader;
}

void FsBlobView::read(void *target, uint64_t offset, uint64_t size) const {
  SharedLock l(_mutex);
  _updateAccessTimestamp();
  _baseBlob->read(target, offset + HEADER_SIZE, size);
}

uint64_t FsBlobView::tryRead(void *target, uint64_t offset, uint64_t size) const {
  SharedLock l(_mutex);
  _updateAccessTimestamp();
  return _baseBlob->tryRead(target, offset + HEADER_SIZE, size);
}

void FsBlobView::write(const void *source, uint64_t offset, uint64_t size) {
  Lock l (_mutex);
  _baseBlob->write(source, offset + HEADER_SIZE, size);
  _metadata._info.size = fspp::num_bytes_t(_baseBlob->size() - HEADER_SIZE);
  _updateModificationTimestamp();
  _updateChangeTimestamp();
}

void FsBlobView::updateModificationTimestamp() {
  Lock l(_mutex);
  _updateModificationTimestamp();
}


void FsBlobView::_updateModificationTimestamp() {
  _metadata._info.mtime = cpputils::time::now();
  _storeMetadata();
}

void FsBlobView::updateChangeTimestamp() {
  Lock l(_mutex);
  _updateChangeTimestamp();
}

void FsBlobView::_updateChangeTimestamp() {
  _metadata._info.ctime = cpputils::time::now();
  _storeMetadata();
}

void FsBlobView::utimens(timespec atime, timespec mtime) {
  Lock l(_mutex);
  _metadata._info.atime = atime;
  _metadata._info.mtime = mtime;
  _updateChangeTimestamp();
  _storeMetadata();
}

void FsBlobView::updateAccessTimestamp() const {
  Lock l(_mutex);
  // TODO: proper implementation
  if (_timestampUpdateBehavior != fsblobstore::TimestampUpdateBehavior::NOATIME) {
    _updateAccessTimestamp();
  }
}

void FsBlobView::_updateAccessTimestamp() const {
  _metadata._info.atime = cpputils::time::now();
  _storeMetadata();
}

uint16_t FsBlobView::getFormatVersionHeader(const blobstore::Blob &blob) {
  static_assert(sizeof(uint16_t) == sizeof(FORMAT_VERSION_HEADER), "Wrong type used to read format version header");
  uint16_t actualFormatVersion;
  blob.read(&actualFormatVersion, 0, sizeof(FORMAT_VERSION_HEADER));
  return actualFormatVersion;
}

void FsBlobView::_checkHeader(const blobstore::Blob &blob) {
  uint16_t actualFormatVersion = getFormatVersionHeader(blob);
  if (FORMAT_VERSION_HEADER != actualFormatVersion) {
    throw std::runtime_error("This file system entity has the wrong format. Was it created with a newer version of CryFS?");
  }
}

FsBlobView::BlobType FsBlobView::_metadataToBlobtype(const Metadata& metadata) {
  const auto& m = metadata._info.mode;
  if (m.hasDirFlag()) {
    return BlobType::DIR;
  } else if (m.hasFileFlag()) {
    return BlobType::FILE;
  } else if (m.hasSymlinkFlag()){
    return BlobType::SYMLINK;
  } else {
    throw std::runtime_error("Illegal Blob Type");
  }
}

void FsBlobView::chmod(fspp::mode_t mode) {
  Lock l(_mutex);
  _updateChangeTimestamp();
  _metadata._info.mode = _metadata._info.mode.changePermissions(mode);
  _storeMetadata();
}

fspp::stat_info FsBlobView::stat() {
  Lock l(_mutex);
  //updateAccessTimestamp();
  if (blobType() == BlobType::DIR) {
    _metadata._info.size = DIR_LSTAT_SIZE;
  } else {
    _metadata._info.size = fspp::num_bytes_t(_baseBlob->size() - HEADER_SIZE);
  }
  return _metadata._info;
}

void FsBlobView::chown(fspp::uid_t uid, fspp::gid_t gid) {
  Lock l(_mutex);
  _updateChangeTimestamp();
  if (uid != fspp::uid_t(-1)) {
    _metadata._info.uid = uid;
  }
  if (gid != fspp::gid_t(-1)) {
    _metadata._info.gid = gid;
  }
  _storeMetadata();
}

void FsBlobView::link() {
  Lock l(_mutex);
  _updateChangeTimestamp();
  _metadata._info.nlink += 1;
  _storeMetadata();
}

bool FsBlobView::unlink() {
  Lock l(_mutex);
  _updateChangeTimestamp();
  ASSERT(_metadata._info.nlink != 0, "Unlink called on an FsBlobView that already had a link count of 0! This should never happen");
  _metadata._info.nlink -= 1;
  _storeMetadata();
  return _metadata._info.nlink == 0;
}

FsBlobView::Metadata::Metadata(uint32_t nlink, fspp::mode_t mode, fspp::uid_t uid, fspp::gid_t gid, fspp::num_bytes_t size, timespec atime, timespec mtime, timespec ctime)
{
  _info.nlink = nlink;
  _info.mode = mode;
  _info.uid = uid;
  _info.gid = gid;
  _info.size = size;
  _info.blocks = 1;
  _info.atime = atime;
  _info.mtime = mtime;
  _info.ctime = ctime;
}


// TODO: (joka921): review if this makes a difference
FsBlobView::Metadata FsBlobView::Metadata::rootMetaData() {
  fspp::stat_info result;
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
  result.size = fspp::num_bytes_t(FsBlobView::DIR_LSTAT_SIZE);

  result.nlink = 2;
  struct timespec now = cpputils::time::now();
  result.atime = now;
  result.mtime = now;
  result.ctime = now;
  Metadata meta;
  meta._info = result;
  return meta;
}

}
