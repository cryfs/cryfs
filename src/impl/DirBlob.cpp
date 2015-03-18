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

//TODO Refactor: Keep a parsed dir structure (list of entries and blob keys they're pointing to) in memory and serialize/deserialize it

namespace cryfs {

DirBlob::DirBlob(unique_ptr<Blob> blob)
: _blob(std::move(blob)) {
  assert(magicNumber() == MagicNumbers::DIR);
}

DirBlob::~DirBlob() {
}

unique_ptr<DirBlob> DirBlob::InitializeEmptyDir(unique_ptr<Blob> blob) {
  blob->resize(1);
  unsigned char magicNumber = MagicNumbers::DIR;
  blob->write(&magicNumber, 0, 1);
  return make_unique<DirBlob>(std::move(blob));
}

unsigned char DirBlob::magicNumber() const {
  unsigned char number;
  _blob->read(&number, 0, 1);
  return number;
}

unique_ptr<vector<fspp::Dir::Entry>> DirBlob::GetChildren() const {
  Data entries(_blob->size()-1);
  _blob->read(entries.data(), 1, _blob->size()-1);

  auto result = make_unique<vector<fspp::Dir::Entry>>();

  const char *pos = (const char*)entries.data();
  while(pos < (const char*)entries.data()+entries.size()) {
    pos = readAndAddNextChild(pos, result.get());
  }

  return result;
}

bool DirBlob::hasChild(const string &name) const {
  //TODO Faster implementation without creating children array possible
  auto children = GetChildren();
  for (const auto &child : *children) {
	if (child.name == name) {
      return true;
	}
  }
  return false;
}

const char *DirBlob::readAndAddNextChild(const char *pos, vector<fspp::Dir::Entry> *result) const {
  // Read type magic number (whether it is a dir or a file)
  fspp::Dir::EntryType type = static_cast<fspp::Dir::EntryType>(*reinterpret_cast<const unsigned char*>(pos));
  pos += 1;

  size_t length = strlen(pos);
  std::string name(pos, length);
  result->emplace_back(fspp::Dir::Entry(type, name));
  const char *posAfterName = pos + length + 1;
  const char *posAfterKey = posAfterName + strlen(posAfterName) + 1;
  return posAfterKey;
}

void DirBlob::AddChildDir(const std::string &name, const Key &blobKey) {
  AddChild(name, blobKey, fspp::Dir::EntryType::DIR);
}

void DirBlob::AddChildFile(const std::string &name, const Key &blobKey) {
  AddChild(name, blobKey, fspp::Dir::EntryType::FILE);
}

void DirBlob::AddChild(const std::string &name, const Key &blobKey, fspp::Dir::EntryType entryType) {
  if (hasChild(name)) {
	throw fspp::fuse::FuseErrnoException(EEXIST);
  }

  //TODO blob.resize(blob.size()+X) has to traverse tree twice. Better would be blob.addSize(X) which returns old size
  uint64_t oldBlobSize = _blob->size();
  string blobKeyStr = blobKey.ToString();
  _blob->resize(oldBlobSize + name.size() + 1 + blobKeyStr.size() + 1);

  //Write entry type
  unsigned char entryTypeMagicNumber = static_cast<unsigned char>(entryType);
  _blob->write(&entryTypeMagicNumber, oldBlobSize, 1);
  //Write entry name inclusive null terminator
  _blob->write(name.c_str(), oldBlobSize + 1, name.size()+1);
  //Write blob key inclusive null terminator
  _blob->write(blobKeyStr.c_str(), oldBlobSize + 1 + name.size() + 1, blobKeyStr.size()+1);
}

pair<fspp::Dir::EntryType, Key> DirBlob::GetChild(const string &name) const {
  Data entries(_blob->size()-1);
  _blob->read(entries.data(), 1, _blob->size()-1);

  const char *pos = (const char*)entries.data();
  while(pos < (const char*)entries.data()+entries.size()) {
    fspp::Dir::EntryType type = static_cast<fspp::Dir::EntryType>(*reinterpret_cast<const unsigned char*>(pos));
    pos += 1; // Skip entry type magic number (whether it is a dir or a file)
    size_t name_length = strlen(pos);
    if (name_length == name.size() && 0==std::memcmp(pos, name.c_str(), name_length)) {
      pos += strlen(pos) + 1; // Skip name
      return make_pair(type, Key::FromString(pos)); // Return key
    }
    pos += strlen(pos) + 1; // Skip name
    pos += strlen(pos) + 1; // Skip key
  }
  throw fspp::fuse::FuseErrnoException(ENOENT);
}


}
