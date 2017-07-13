#pragma once
#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_ONDISK_ONDISKBLOCKSTORE2_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_ONDISK_ONDISKBLOCKSTORE2_H_

#include "../../interface/BlockStore2.h"
#include <boost/filesystem/path.hpp>
#include <cpp-utils/macros.h>
#include <cpp-utils/pointer/unique_ref.h>
#include "OnDiskBlockStore.h"
#include <cpp-utils/logging/logging.h>
#include <sys/statvfs.h>

namespace blockstore {
namespace ondisk {

class OnDiskBlockStore2 final: public BlockStore2 {
public:
  explicit OnDiskBlockStore2(const boost::filesystem::path& path);

  boost::future<bool> tryCreate(const Key &key, const cpputils::Data &data) override;
  boost::future<bool> remove(const Key &key) override;
  boost::future<boost::optional<cpputils::Data>> load(const Key &key) const override;
  boost::future<void> store(const Key &key, const cpputils::Data &data) override;
  uint64_t numBlocks() const override;
  uint64_t estimateNumFreeBytes() const override;
  uint64_t blockSizeFromPhysicalBlockSize(uint64_t blockSize) const override;
  void forEachBlock(std::function<void (const Key &)> callback) const override;

private:
  boost::filesystem::path _rootDir;

  static const std::string FORMAT_VERSION_HEADER_PREFIX;
  static const std::string FORMAT_VERSION_HEADER;

  boost::filesystem::path _getFilepath(const Key &key) const;
  static cpputils::Data _checkAndRemoveHeader(const cpputils::Data &data);
  static bool _isAcceptedCryfsHeader(const cpputils::Data &data);
  static bool _isOtherCryfsHeader(const cpputils::Data &data);
  static unsigned int formatVersionHeaderSize();

  DISALLOW_COPY_AND_ASSIGN(OnDiskBlockStore2);
};

}
}

#endif
