#include "DirBlob.h"
#include <cassert>

//TODO Remove and replace with exception hierarchy
#include "messmer/fspp/fuse/FuseErrnoException.h"

#include <messmer/cpp-utils/data/Data.h>
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

DirBlob::DirBlob(unique_ref<Blob> blob, CryDevice *device) :
    _device(device), _blob(std::move(blob)), _entries(), _changed(false) {
  ASSERT(magicNumber() == MagicNumbers::DIR, "Loaded blob is not a directory");
  _readEntriesFromBlob();
}

DirBlob::~DirBlob() {
  _writeEntriesToBlob();
}

void DirBlob::flush() {
  _writeEntriesToBlob();
  _blob->flush();
}

unique_ref<DirBlob> DirBlob::InitializeEmptyDir(unique_ref<Blob> blob, CryDevice *device) {
  blob->resize(1);
  unsigned char magicNumber = MagicNumbers::DIR;
  blob->write(&magicNumber, 0, 1);
  return make_unique_ref<DirBlob>(std::move(blob), device);
}

unsigned char DirBlob::magicNumber() const {
  unsigned char number;
  _blob->read(&number, 0, 1);
  return number;
}

void DirBlob::_writeEntriesToBlob() {
  if (_changed) {
    //TODO Resizing is imperformant
    _blob->resize(1);
    unsigned int offset = 1;
    for (const auto &entry : _entries) {
	  uint8_t entryTypeMagicNumber = static_cast<uint8_t>(entry.type);
	  _blob->write(&entryTypeMagicNumber, offset, 1);
	  offset += 1;
	  _blob->write(entry.name.c_str(), offset, entry.name.size() + 1);
	  offset += entry.name.size() + 1;
	  string keystr = entry.key.ToString();
	  _blob->write(keystr.c_str(), offset, keystr.size() + 1);
	  offset += keystr.size() + 1;
	  _blob->write(&entry.uid, offset, sizeof(uid_t));
	  //TODO Writing them all in separate write calls is maybe imperformant. We could write the whole entry in one write call instead.
	  offset += sizeof(uid_t);
    _blob->write(&entry.gid, offset, sizeof(gid_t));
    offset += sizeof(gid_t);
    _blob->write(&entry.mode, offset, sizeof(mode_t));
    offset += sizeof(mode_t);
    }
    _changed = false;
  }
}

void DirBlob::_readEntriesFromBlob() {
  _entries.clear();
  Data data(_blob->size() - 1);
  _blob->read(data.data(), 1, _blob->size() - 1);

  const char *pos = (const char*) data.data();
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

bool DirBlob::hasChild(const string &name) const {
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
  if (hasChild(name)) {
    throw fspp::fuse::FuseErrnoException(EEXIST);
  }

  _entries.emplace_back(entryType, name, blobKey, mode, uid, gid);
  _changed = true;
}

const DirBlob::Entry &DirBlob::GetChild(const string &name) const {
  auto found = std::find_if(_entries.begin(), _entries.end(), [name] (const Entry &entry) {
    return entry.name == name;
  });
  if (found == _entries.end()) {
    throw fspp::fuse::FuseErrnoException(ENOENT);
  }
  return *found;
}

const DirBlob::Entry &DirBlob::GetChild(const Key &key) const {
  auto found = std::find_if(_entries.begin(), _entries.end(), [key] (const Entry &entry) {
	return entry.key == key;
  });
  if (found == _entries.end()) {
	throw fspp::fuse::FuseErrnoException(ENOENT);
  }
  return *found;
}

void DirBlob::RemoveChild(const Key &key) {
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
  result->reserve(result->size() + _entries.size());
  for (const auto &entry : _entries) {
    result->emplace_back(entry.type, entry.name);
  }
}

void DirBlob::statChild(const Key &key, struct ::stat *result) const {
  auto child = GetChild(key);
  //TODO Loading the blob for only getting the size of the file/symlink is not very performant.
  //     Furthermore, this is the only reason why DirBlob needs a pointer to CryDevice, which is ugly
  result->st_mode = child.mode;
  result->st_uid = child.uid;
  result->st_gid = child.gid;
  //TODO If possible without performance loss, then for a directory, st_nlink should return number of dir entries (including "." and "..")
  result->st_nlink = 1;
  //TODO Handle file access times
  result->st_mtime = result->st_ctime = result->st_atime = 0;
  if (child.type == fspp::Dir::EntryType::FILE) {
    auto blob = _device->LoadBlob(key);
    if (blob == none) {
      //TODO Log error
    } else {
      result->st_size = FileBlob(std::move(*blob)).size();
    }
  } else if (child.type == fspp::Dir::EntryType::DIR) {
	//TODO Why do dirs have 4096 bytes in size? Does that make sense?
    result->st_size = 4096;
  } else if (child.type == fspp::Dir::EntryType::SYMLINK) {
	//TODO Necessary with fuse or does fuse set this on symlinks anyhow?
    auto blob = _device->LoadBlob(key);
    if (blob == none) {
      //TODO Log error
    } else {
      result->st_size = SymlinkBlob(std::move(*blob)).target().native().size();
    }
  } else {
	ASSERT(false, "Unknown child type");
  }
  //TODO Move ceilDivision to general utils which can be used by cryfs as well
  result->st_blocks = blobstore::onblocks::utils::ceilDivision(result->st_size, 512);
  result->st_blksize = _device->BLOCKSIZE_BYTES;
}

void DirBlob::chmodChild(const Key &key, mode_t mode) {
  auto found = _findChild(key);
  ASSERT ((S_ISREG(mode) && S_ISREG(found->mode)) || (S_ISDIR(mode) && S_ISDIR(found->mode)) || (S_ISLNK(mode)), "Unknown mode in entry");
  found->mode = mode;
  _changed = true;
}

void DirBlob::chownChild(const Key &key, uid_t uid, gid_t gid) {
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

}
