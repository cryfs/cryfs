#pragma once
#ifndef CRYFS_LIB_IMPL_FILEBLOCK_H_
#define CRYFS_LIB_IMPL_FILEBLOCK_H_

#include <blockstore/interface/Block.h>
#include <memory>

namespace cryfs {

class FileBlock {
public:
  FileBlock(std::unique_ptr<blockstore::Block> block);
  virtual ~FileBlock();

  static bool IsFile(const blockstore::Block &block);

  void InitializeEmptyFile();

private:
  std::unique_ptr<blockstore::Block> _block;

  unsigned char *magicNumber();
  static const unsigned char *magicNumber(const blockstore::Block &block);
};

}

#endif
