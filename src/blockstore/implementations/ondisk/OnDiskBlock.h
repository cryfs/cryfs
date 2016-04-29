#pragma once
#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_ONDISK_ONDISKBLOCK_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_ONDISK_ONDISKBLOCK_H_

#include <boost/filesystem/path.hpp>
#include "../../interface/Block.h"
#include <cpp-utils/data/Data.h>
#include <iostream>

#include <cpp-utils/pointer/unique_ref.h>
#include <mutex>

namespace blockstore {
namespace ondisk {
class OnDiskBlockStore;

class OnDiskBlock final: public Block {
public:
  OnDiskBlock(const Key &key, const boost::filesystem::path &filepath, cpputils::Data data);
  ~OnDiskBlock();

  static const std::string FORMAT_VERSION_HEADER_PREFIX;
  static const std::string FORMAT_VERSION_HEADER;
  static unsigned int formatVersionHeaderSize();
  static uint64_t blockSizeFromPhysicalBlockSize(uint64_t blockSize);

  static boost::optional<cpputils::unique_ref<OnDiskBlock>> LoadFromDisk(const boost::filesystem::path &rootdir, const Key &key);
  static boost::optional<cpputils::unique_ref<OnDiskBlock>> CreateOnDisk(const boost::filesystem::path &rootdir, const Key &key, cpputils::Data data);
  static void RemoveFromDisk(const boost::filesystem::path &rootdir, const Key &key);

  const void *data() const override;
  void write(const void *source, uint64_t offset, uint64_t size) override;

  void flush() override;

  size_t size() const override;
  void resize(size_t newSize) override;

private:

  static bool _isAcceptedCryfsHeader(const cpputils::Data &data);
  static bool _isOtherCryfsHeader(const cpputils::Data &data);
  static void _checkHeader(std::istream *str);
  static boost::filesystem::path _getFilepath(const boost::filesystem::path &rootdir, const Key &key);

  const boost::filesystem::path _filepath;
  cpputils::Data _data;
  bool _dataChanged;

  static boost::optional<cpputils::Data> _loadFromDisk(const boost::filesystem::path &filepath);
  void _storeToDisk() const;

  std::mutex _mutex;

  DISALLOW_COPY_AND_ASSIGN(OnDiskBlock);
};

}
}

#endif
