#pragma once
#ifndef BLOCKSTORE_IMPLEMENTATIONS_ONDISK_ONDISKBLOCK_H_
#define BLOCKSTORE_IMPLEMENTATIONS_ONDISK_ONDISKBLOCK_H_

#include <boost/filesystem/path.hpp>
#include <messmer/blockstore/interface/Block.h>
#include <messmer/blockstore/utils/Data.h>
#include <iostream>

#include "messmer/cpp-utils/macros.h"

namespace blockstore {
namespace ondisk {
class OnDiskBlockStore;

class OnDiskBlock: public Block {
public:
  virtual ~OnDiskBlock();

  static std::unique_ptr<OnDiskBlock> LoadFromDisk(const boost::filesystem::path &rootdir, const Key &key);
  static std::unique_ptr<OnDiskBlock> CreateOnDisk(const boost::filesystem::path &rootdir, const Key &key, size_t size);

  void *data() override;
  const void *data() const override;

  void flush() override;

  size_t size() const override;

private:
  const boost::filesystem::path _filepath;
  Data _data;

  OnDiskBlock(const Key &key, const boost::filesystem::path &filepath, size_t size);
  OnDiskBlock(const Key &key, const boost::filesystem::path &filepath, Data &&data);

  void _fillDataWithZeroes();
  void _storeToDisk() const;

  DISALLOW_COPY_AND_ASSIGN(OnDiskBlock);
};

}
}

#endif
