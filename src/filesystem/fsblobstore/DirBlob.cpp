#include "DirBlob.h"
#include <cassert>

//TODO Remove and replace with exception hierarchy
#include "messmer/fspp/fuse/FuseErrnoException.h"

#include <messmer/blobstore/implementations/onblocks/utils/Math.h>
#include "MagicNumbers.h"
#include "../CryDevice.h"
#include "FileBlob.h"
#include "SymlinkBlob.h"

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

//TODO Factor out a DirEntryList class

DirBlob::DirBlob(unique_ref<Blob> blob, std::function<off_t (const blockstore::Key&)> getLstatSize) :
    FsBlob(std::move(blob)), _getLstatSize(getLstatSize), _entries(), _changed(false) {
  ASSERT(magicNumber() == MagicNumbers::DIR, "Loaded blob is not a directory");
  _readEntriesFromBlob();
}

DirBlob::~DirBlob() {
  _writeEntriesToBlob();
}

void DirBlob::flush() {
  _writeEntriesToBlob();
  baseBlob().flush();
}

unique_ref<DirBlob> DirBlob::InitializeEmptyDir(unique_ref<Blob> blob, std::function<off_t(const blockstore::Key&)> getLstatSize) {
  InitializeBlobWithMagicNumber(blob.get(), MagicNumbers::DIR);
  return make_unique_ref<DirBlob>(std::move(blob), getLstatSize);
}

size_t DirBlob::_serializedSizeOfEntry(const DirBlob::Entry &entry) {
  return 1 + (entry.name.size() + 1) + (entry.key.STRING_LENGTH + 1) + sizeof(uid_t) + sizeof(gid_t) + sizeof(mode_t);
}

void DirBlob::_serializeEntry(const DirBlob::Entry & entry, uint8_t *dest) {
  //TODO Write key as binary?
  string keystr = entry.key.ToString();

  unsigned int offset = 0;
  *(dest+offset) = static_cast<uint8_t>(entry.type);
  offset += 1;

  std::memcpy(dest+offset, entry.name.c_str(), entry.name.size()+1);
  offset += entry.name.size() + 1;

  std::memcpy(dest+offset, keystr.c_str(), keystr.size() + 1);
  offset += keystr.size() + 1;

  *reinterpret_cast<uid_t*>(dest+offset) = entry.uid;
  offset += sizeof(uid_t);

  *reinterpret_cast<gid_t*>(dest+offset) = entry.gid;
  offset += sizeof(gid_t);

  *reinterpret_cast<mode_t*>(dest+offset) = entry.mode;
  offset += sizeof(mode_t);

  ASSERT(offset == _serializedSizeOfEntry(entry), "Didn't write correct number of elements");
}

void DirBlob::_writeEntriesToBlob() {
  std::unique_lock<std::mutex> lock(_mutex);
  if (_changed) {
    size_t serializedSize = 0;
    for (const auto &entry : _entries) {
      serializedSize += _serializedSizeOfEntry(entry);
    }
    Data serialized(serializedSize);
    unsigned int offset = 0;
    for (const auto &entry : _entries) {
      _serializeEntry(entry, static_cast<uint8_t*>(serialized.dataOffset(offset)));
      offset += _serializedSizeOfEntry(entry);
    }
    baseBlob().resize(1 + serializedSize);
    baseBlob().write(serialized.data(), 1, serializedSize);
    _changed = false;
  }
}

void DirBlob::_readEntriesFromBlob() {
  //No lock needed, because this is only called from the constructor.
  _entries.clear();
  Data data = baseBlob().readAll();

  const char *pos = (const char*)data.data() + 1; // +1 for magic number of blob
  while (pos < (const char*) data.data() + data.size()) {
    pos = readAndAddNextChild(pos, &_entries);
  }
}

const char *DirBlob::readAndAddNextChild(const char *pos,
    vector<DirBlob::Entry> *result) const {
  // Read type magic number (whether it is a dir or a file)
  fspp::Dir::EntryType type =
      static_cast<fspp::Dir::EntryType>(*reinterpret_cast<const unsigned char*>(pos));
  pos += 1;

  size_t namelength = strlen(pos);
  std::string name(pos, namelength);
  pos += namelength + 1;

  size_t keylength = strlen(pos);
  std::string keystr(pos, keylength);
  pos += keylength + 1;

  //It might make sense to read all of them at once with a memcpy

  uid_t uid = *(uid_t*)pos;
  pos += sizeof(uid_t);

  gid_t gid = *(gid_t*)pos;
  pos += sizeof(gid_t);

  mode_t mode = *(mode_t*)pos;
  pos += sizeof(mode_t);

  result->emplace_back(type, name, Key::FromString(keystr), mode, uid, gid);
  return pos;
}

bool DirBlob::_hasChild(const string &name) const {
  auto found = std::find_if(_entries.begin(), _entries.end(), [name] (const Entry &entry) {
    return entry.name == name;
  });
  return found != _entries.end();
}

void DirBlob::AddChildDir(const std::string &name, const Key &blobKey, mode_t mode, uid_t uid, gid_t gid) {
  AddChild(name, blobKey, fspp::Dir::EntryType::DIR, mode, uid, gid);
}

void DirBlob::AddChildFile(const std::string &name, const Key &blobKey, mode_t mode, uid_t uid, gid_t gid) {
  AddChild(name, blobKey, fspp::Dir::EntryType::FILE, mode, uid, gid);
}

void DirBlob::AddChildSymlink(const std::string &name, const blockstore::Key &blobKey, uid_t uid, gid_t gid) {
  AddChild(name, blobKey, fspp::Dir::EntryType::SYMLINK, S_IFLNK | S_IRUSR | S_IWUSR | S_IXUSR | S_IRGRP | S_IWGRP | S_IXGRP | S_IROTH | S_IWOTH | S_IXOTH, uid, gid);
}

void DirBlob::AddChild(const std::string &name, const Key &blobKey,
    fspp::Dir::EntryType entryType, mode_t mode, uid_t uid, gid_t gid) {
  std::unique_lock<std::mutex> lock(_mutex);
  if (_hasChild(name)) {
    throw fspp::fuse::FuseErrnoException(EEXIST);
  }

  _entries.emplace_back(entryType, name, blobKey, mode, uid, gid);
  _changed = true;
}

const DirBlob::Entry &DirBlob::GetChild(const string &name) const {
  std::unique_lock<std::mutex> lock(_mutex);
  auto found = std::find_if(_entries.begin(), _entries.end(), [name] (const Entry &entry) {
    return entry.name == name;
  });
  if (found == _entries.end()) {
    throw fspp::fuse::FuseErrnoException(ENOENT);
  }
  return *found;
}

const DirBlob::Entry &DirBlob::GetChild(const Key &key) const {
  std::unique_lock<std::mutex> lock(_mutex);
  auto found = std::find_if(_entries.begin(), _entries.end(), [key] (const Entry &entry) {
	return entry.key == key;
  });
  if (found == _entries.end()) {
	throw fspp::fuse::FuseErrnoException(ENOENT);
  }
  return *found;
}

void DirBlob::RemoveChild(const Key &key) {
  std::unique_lock<std::mutex> lock(_mutex);
  auto found = _findChild(key);
  _entries.erase(found);
  _changed = true;
}

std::vector<DirBlob::Entry>::iterator DirBlob::_findChild(const Key &key) {
  //TODO Code duplication with GetChild(key)
  auto found = std::find_if(_entries.begin(), _entries.end(), [key] (const Entry &entry) {
	return entry.key == key;
  });
  if (found == _entries.end()) {
	throw fspp::fuse::FuseErrnoException(ENOENT);
  }
  return found;
}

void DirBlob::AppendChildrenTo(vector<fspp::Dir::Entry> *result) const {
  std::unique_lock<std::mutex> lock(_mutex);
  result->reserve(result->size() + _entries.size());
  for (const auto &entry : _entries) {
    result->emplace_back(entry.type, entry.name);
  }
}

off_t DirBlob::lstat_size() const {
  //TODO Why do dirs have 4096 bytes in size? Does that make sense?
  return 4096;
}

void DirBlob::statChild(const Key &key, struct ::stat *result) const {
  auto child = GetChild(key);
  //TODO Loading the blob for only getting the size of the file/symlink is not very performant.
  //     Furthermore, this is the only reason why DirBlob needs a pointer to _fsBlobStore, which is ugly
  result->st_mode = child.mode;
  result->st_uid = child.uid;
  result->st_gid = child.gid;
  //TODO If possible without performance loss, then for a directory, st_nlink should return number of dir entries (including "." and "..")
  result->st_nlink = 1;
  //TODO Handle file access times
  result->st_mtime = result->st_ctime = result->st_atime = 0;
  result->st_size = _getLstatSize(key);
  //TODO Move ceilDivision to general utils which can be used by cryfs as well
  result->st_blocks = blobstore::onblocks::utils::ceilDivision(result->st_size, 512);
  result->st_blksize = CryDevice::BLOCKSIZE_BYTES; //TODO FsBlobStore::BLOCKSIZE_BYTES would be cleaner
}

void DirBlob::chmodChild(const Key &key, mode_t mode) {
  std::unique_lock<std::mutex> lock(_mutex);
  auto found = _findChild(key);
  ASSERT ((S_ISREG(mode) && S_ISREG(found->mode)) || (S_ISDIR(mode) && S_ISDIR(found->mode)) || (S_ISLNK(mode)), "Unknown mode in entry");
  found->mode = mode;
  _changed = true;
}

void DirBlob::chownChild(const Key &key, uid_t uid, gid_t gid) {
  std::unique_lock<std::mutex> lock(_mutex);
  auto found = _findChild(key);
  if (uid != (uid_t)-1) {
    found->uid = uid;
    _changed = true;
  }
  if (gid != (gid_t)-1) {
    found->gid = gid;
    _changed = true;
  }
}

void DirBlob::setLstatSizeGetter(std::function<off_t(const blockstore::Key&)> getLstatSize) {
    std::unique_lock<std::mutex> lock(_mutex);
    _getLstatSize = getLstatSize;
}

}
}
