#include "FileBlock.h"

#include "MagicNumbers.h"

using std::unique_ptr;
using blockstore::Block;

namespace cryfs {

FileBlock::FileBlock(unique_ptr<Block> block)
: _block(std::move(block)) {
}

FileBlock::~FileBlock() {
}

void FileBlock::InitializeEmptyFile() {
  *magicNumber() = MagicNumbers::FILE;
}

unsigned char *FileBlock::magicNumber() {
  return const_cast<unsigned char*>(magicNumber(const_cast<const Block&>(*_block)));
}

const unsigned char *FileBlock::magicNumber(const blockstore::Block &block) {
  return (unsigned char*)block.data();
}

bool FileBlock::IsFile(const Block &block) {
  return *magicNumber(block) == MagicNumbers::FILE;
}

}
