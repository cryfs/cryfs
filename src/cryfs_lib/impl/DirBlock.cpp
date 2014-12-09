#include "DirBlock.h"

#include <cassert>

//TODO Remove and replace with exception hierarchy
#include "fspp/fuse/FuseErrnoException.h"

#include "MagicNumbers.h"

using std::unique_ptr;
using std::make_unique;
using std::vector;
using std::string;

using blockstore::Block;

namespace cryfs {

DirBlock::DirBlock(unique_ptr<Block> block)
: _block(std::move(block)) {
}

DirBlock::~DirBlock() {
}

void DirBlock::InitializeEmptyDir() {
  *magicNumber() = MagicNumbers::DIR;
  *entryCounter() = 0;
}

unsigned char *DirBlock::magicNumber() {
  return const_cast<unsigned char*>(magicNumber(const_cast<const Block&>(*_block)));
}

const unsigned char *DirBlock::magicNumber(const blockstore::Block &block) {
  return (unsigned char*)block.data();
}

bool DirBlock::IsDir(const blockstore::Block &block) {
  return *magicNumber(block) == MagicNumbers::DIR;
}

unique_ptr<vector<string>> DirBlock::GetChildren() const {
  auto result = make_unique<vector<string>>();
  unsigned int entryCount = *entryCounter();
  result->reserve(entryCount);

  const char *pos = entriesBegin();
  for (unsigned int i = 0; i < entryCount; ++i) {
    pos = readAndAddNextChild(pos, result.get());
  }

  return result;
}

const char *DirBlock::readAndAddNextChild(const char *pos, vector<string> *result) const {
  size_t length = strlen(pos);
  result->emplace_back(pos, length);
  const char *posAfterName = pos + length + 1;
  const char *posAfterKey = posAfterName + strlen(posAfterName) + 1;
  return posAfterKey;
}

void DirBlock::AddChild(const std::string &name, const std::string &blockKey) {
  char *insertPos = entriesEnd();
  assertEnoughSpaceLeft(insertPos, name.size() + 1 + blockKey.size() + 1);

  memcpy(insertPos, name.c_str(), name.size()+1);
  memcpy(insertPos + name.size()+1, blockKey.c_str(), blockKey.size()+1);
  ++(*entryCounter());
}

void DirBlock::assertEnoughSpaceLeft(char *insertPos, size_t insertSize) const {
  size_t usedSize = insertPos - (char*)_block->data();
  assert(usedSize + insertSize <= _block->size());
}

string DirBlock::GetBlockKeyForName(const string &name) const {
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

unsigned int *DirBlock::entryCounter() {
  return const_cast<unsigned int*>(const_cast<const DirBlock*>(this)->entryCounter());
}

const unsigned int *DirBlock::entryCounter() const {
  return (unsigned int*)((char*)_block->data() + sizeof(unsigned char));
}

char *DirBlock::entriesBegin() {
  return const_cast<char*>(const_cast<const DirBlock*>(this)->entriesBegin());
}

const char *DirBlock::entriesBegin() const {
  return (char *)(_block->data())+sizeof(unsigned char) + sizeof(unsigned int);
}

char *DirBlock::entriesEnd() {
  unsigned int entryCount = *entryCounter();
  char *pos = entriesBegin();
  for(unsigned int i = 0; i < entryCount; ++i) {
    pos += strlen(pos) + 1; // Skip entry name
    pos += strlen(pos) + 1; // Skip entry key
  }
  return pos;
}

}
