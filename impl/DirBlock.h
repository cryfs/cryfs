#pragma once
#ifndef CRYFS_LIB_IMPL_DIRBLOCK_H_
#define CRYFS_LIB_IMPL_DIRBLOCK_H_

#include <messmer/blockstore/interface/Block.h>
#include <messmer/blockstore/utils/Key.h>
#include "messmer/cpp-utils/macros.h"

#include <memory>
#include <vector>

namespace cryfs{

class DirBlock {
public:
  DirBlock(std::unique_ptr<blockstore::Block> block);
  virtual ~DirBlock();

  void InitializeEmptyDir();
  std::unique_ptr<std::vector<std::string>> GetChildren() const;
  void AddChild(const std::string &name, const blockstore::Key &blockKey);
  blockstore::Key GetBlockKeyForName(const std::string &name) const;

  static bool IsDir(const blockstore::Block &block);

private:
  unsigned char *magicNumber();
  static const unsigned char *magicNumber(const blockstore::Block &block);
  unsigned int *entryCounter();
  const unsigned int *entryCounter() const;
  char *entriesBegin();
  const char *entriesBegin() const;
  char *entriesEnd();

  const char *readAndAddNextChild(const char *pos, std::vector<std::string> *result) const;
  void assertEnoughSpaceLeft(char *insertPos, size_t insertSize) const;

  std::unique_ptr<blockstore::Block> _block;

  DISALLOW_COPY_AND_ASSIGN(DirBlock);
};

}

#endif
