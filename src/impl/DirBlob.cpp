#include "DirBlob.h"
#include <cassert>

//TODO Remove and replace with exception hierarchy
#include "messmer/fspp/fuse/FuseErrnoException.h"

#include <messmer/blockstore/utils/Data.h>
#include "MagicNumbers.h"

using std::unique_ptr;
using std::make_unique;
using std::vector;
using std::string;
using std::pair;
using std::make_pair;

using blobstore::Blob;
using blockstore::Key;
using blockstore::Data;

namespace cryfs {

DirBlob::DirBlob(unique_ptr<Blob> blob) :
    _blob(std::move(blob)), _entries(), _changed(false) {
  assert(magicNumber() == MagicNumbers::DIR);
  _readEntriesFromBlob();
}

DirBlob::~DirBlob() {
  flush();
}

void DirBlob::flush() {
  if (_changed) {
    _writeEntriesToBlob();
    _changed = false;
  }
  _blob->flush();
}

unique_ptr<DirBlob> DirBlob::InitializeEmptyDir(unique_ptr<Blob> blob) {
  blob->resize(1);
  unsigned char magicNumber = MagicNumbers::DIR;
  blob->write(&magicNumber, 0, 1);
  return make_unique < DirBlob > (std::move(blob));
}

unsigned char DirBlob::magicNumber() const {
  unsigned char number;
  _blob->read(&number, 0, 1);
  return number;
}

void DirBlob::_writeEntriesToBlob() {
  //TODO Resizing is imperformant
  _blob->resize(1);
  unsigned int offset = 1;
  for (const auto &entry : _entries) {
    unsigned char entryTypeMagicNumber = static_cast<unsigned char>(entry.type);
    _blob->write(&entryTypeMagicNumber, offset, 1);
    offset += 1;
    _blob->write(entry.name.c_str(), offset, entry.name.size() + 1);
    offset += entry.name.size() + 1;
    string keystr = entry.key.ToString();
    _blob->write(keystr.c_str(), offset, keystr.size() + 1);
    offset += keystr.size() + 1;
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

  result->emplace_back(type, name, Key::FromString(keystr));
  return pos;
}

bool DirBlob::hasChild(const string &name) const {
  auto found = std::find_if(_entries.begin(), _entries.end(), [name] (const Entry &entry) {
    return entry.name == name;
  });
  return found != _entries.end();
}

void DirBlob::AddChildDir(const std::string &name, const Key &blobKey) {
  AddChild(name, blobKey, fspp::Dir::EntryType::DIR);
}

void DirBlob::AddChildFile(const std::string &name, const Key &blobKey) {
  AddChild(name, blobKey, fspp::Dir::EntryType::FILE);
}

void DirBlob::AddChild(const std::string &name, const Key &blobKey,
    fspp::Dir::EntryType entryType) {
  if (hasChild(name)) {
    throw fspp::fuse::FuseErrnoException(EEXIST);
  }

  _entries.emplace_back(entryType, name, blobKey);
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

void DirBlob::RemoveChild(const Key &key) {
  auto found = std::find_if(_entries.begin(), _entries.end(), [key] (const Entry &entry) {
    return entry.key == key;
  });
  if (found == _entries.end()) {
    throw fspp::fuse::FuseErrnoException(ENOENT);
  }
  _entries.erase(found);
  _changed = true;
}

void DirBlob::AppendChildrenTo(vector<fspp::Dir::Entry> *result) const {
  result->reserve(result->size() + _entries.size());
  for (const auto &entry : _entries) {
    result->emplace_back(entry.type, entry.name);
  }
}

}
