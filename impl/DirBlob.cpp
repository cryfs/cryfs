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

using blobstore::Blob;
using blockstore::Key;
using blockstore::Data;

//TODO Refactor: Keep a parsed dir structure (list of entries and blob keys they're pointing to) in memory and serialize/deserialize it

namespace cryfs {

DirBlob::DirBlob(unique_ptr<Blob> blob)
: _blob(std::move(blob)) {
}

DirBlob::~DirBlob() {
}

void DirBlob::InitializeEmptyDir() {
  _blob->resize(1);
  unsigned char magicNumber = MagicNumbers::DIR;
  _blob->write(&magicNumber, 0, 1);
}

unsigned char DirBlob::magicNumber() const {
  return magicNumber(*_blob);
}

const unsigned char DirBlob::magicNumber(const blobstore::Blob &blob) {
  unsigned char number;
  blob.read(&number, 0, 1);
  return number;
}

bool DirBlob::IsDir(const blobstore::Blob &blob) {
  return magicNumber(blob) == MagicNumbers::DIR;
}

unique_ptr<vector<string>> DirBlob::GetChildren() const {
  Data entries(_blob->size()-1);
  _blob->read(entries.data(), 1, _blob->size()-1);

  auto result = make_unique<vector<string>>();

  const char *pos = (const char*)entries.data();
  while(pos < (const char*)entries.data()+entries.size()) {
    pos = readAndAddNextChild(pos, result.get());
  }

  return result;
}

const char *DirBlob::readAndAddNextChild(const char *pos, vector<string> *result) const {
  size_t length = strlen(pos);
  result->emplace_back(pos, length);
  const char *posAfterName = pos + length + 1;
  const char *posAfterKey = posAfterName + strlen(posAfterName) + 1;
  return posAfterKey;
}

void DirBlob::AddChild(const std::string &name, const Key &blobKey) {
  //TODO blob.resize(blob.size()+X) has to traverse tree twice. Better would be blob.addSize(X) which returns old size
  uint64_t oldBlobSize = _blob->size();
  string blobKeyStr = blobKey.ToString();
  _blob->resize(oldBlobSize + name.size() + 1 + blobKeyStr.size() + 1);

  //Write entry name inclusive null terminator
  _blob->write(name.c_str(), oldBlobSize, name.size()+1);
  //Write blob key inclusive null terminator
  _blob->write(blobKeyStr.c_str(), oldBlobSize + name.size() + 1, blobKeyStr.size()+1);
}

Key DirBlob::GetBlobKeyForName(const string &name) const {
  auto result = make_unique<vector<string>>();

  Data entries(_blob->size()-1);
  _blob->read(entries.data(), 1, _blob->size()-1);

  const char *pos = (const char*)entries.data();
  while(pos < (const char*)entries.data()+entries.size()) {
    size_t name_length = strlen(pos);
    if (name_length == name.size() && 0==std::memcmp(pos, name.c_str(), name_length)) {
      pos += strlen(pos) + 1; // Skip name
      return Key::FromString(pos); // Return key
    }
    pos += strlen(pos) + 1; // Skip name
    pos += strlen(pos) + 1; // Skip key
  }
  throw fspp::fuse::FuseErrnoException(ENOENT);
}


}
