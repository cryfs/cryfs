#include <cryfs_lib/impl/DirBlob.h>

#include <cassert>

//TODO Remove and replace with exception hierarchy
#include "fspp/fuse/FuseErrnoException.h"

#include "MagicNumbers.h"

using std::unique_ptr;
using std::make_unique;
using std::vector;
using std::string;

using blobstore::Blob;

namespace cryfs {

DirBlob::DirBlob(unique_ptr<Blob> blob)
: _blob(std::move(blob)) {
}

DirBlob::~DirBlob() {
}

void DirBlob::InitializeEmptyDir() {
  *magicNumber() = MagicNumbers::DIR;
  *entryCounter() = 0;
}

unsigned char *DirBlob::magicNumber() {
  return const_cast<unsigned char*>(magicNumber(const_cast<const Blob&>(*_blob)));
}

const unsigned char *DirBlob::magicNumber(const blobstore::Blob &blob) {
  return (unsigned char*)blob.data();
}

bool DirBlob::IsDir(const blobstore::Blob &blob) {
  return *magicNumber(blob) == MagicNumbers::DIR;
}

unique_ptr<vector<string>> DirBlob::GetChildren() const {
  auto result = make_unique<vector<string>>();
  unsigned int entryCount = *entryCounter();
  result->reserve(entryCount);

  const char *pos = entriesBegin();
  for (unsigned int i = 0; i < entryCount; ++i) {
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

void DirBlob::AddChild(const std::string &name, const std::string &blobKey) {
  char *insertPos = entriesEnd();
  assertEnoughSpaceLeft(insertPos, name.size() + 1 + blobKey.size() + 1);

  memcpy(insertPos, name.c_str(), name.size()+1);
  memcpy(insertPos + name.size()+1, blobKey.c_str(), blobKey.size()+1);
  ++(*entryCounter());
}

void DirBlob::assertEnoughSpaceLeft(char *insertPos, size_t insertSize) const {
  size_t usedSize = insertPos - (char*)_blob->data();
  assert(usedSize + insertSize <= _blob->size());
}

string DirBlob::GetBlobKeyForName(const string &name) const {
  unsigned int entryCount = *entryCounter();
  const char *pos = entriesBegin();
  for(unsigned int i = 0; i < entryCount; ++i) {
    size_t name_length = strlen(pos);
    if (name_length == name.size() && 0==std::memcmp(pos, name.c_str(), name_length)) {
      pos += strlen(pos) + 1; // Skip name
      return pos; // Return key
    }
    pos += strlen(pos) + 1; // Skip name
    pos += strlen(pos) + 1; // Skip key
  }
  throw fspp::fuse::FuseErrnoException(ENOENT);
}

unsigned int *DirBlob::entryCounter() {
  return const_cast<unsigned int*>(const_cast<const DirBlob*>(this)->entryCounter());
}

const unsigned int *DirBlob::entryCounter() const {
  return (unsigned int*)((char*)_blob->data() + sizeof(unsigned char));
}

char *DirBlob::entriesBegin() {
  return const_cast<char*>(const_cast<const DirBlob*>(this)->entriesBegin());
}

const char *DirBlob::entriesBegin() const {
  return (char *)(_blob->data())+sizeof(unsigned char) + sizeof(unsigned int);
}

char *DirBlob::entriesEnd() {
  unsigned int entryCount = *entryCounter();
  char *pos = entriesBegin();
  for(unsigned int i = 0; i < entryCount; ++i) {
    pos += strlen(pos) + 1; // Skip entry name
    pos += strlen(pos) + 1; // Skip entry key
  }
  return pos;
}

}
