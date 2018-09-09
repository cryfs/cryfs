#pragma once
#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_ONDISK_ONDISKBLOCKSTORE2_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_ONDISK_ONDISKBLOCKSTORE2_H_

#include "../../interface/BlockStore2.h"
#include <boost/filesystem/path.hpp>
#include <cpp-utils/macros.h>
#include <cpp-utils/pointer/unique_ref.h>
#include <cpp-utils/logging/logging.h>

namespace blockstore {
namespace ondisk {

class OnDiskBlockStore2 final: public BlockStore2 {
public:
  explicit OnDiskBlockStore2(const boost::filesystem::path& path);

  bool tryCreate(const BlockId &blockId, const cpputils::Data &data) override;
  bool remove(const BlockId &blockId) override;
  boost::optional<cpputils::Data> load(const BlockId &blockId) const override;
  void store(const BlockId &blockId, const cpputils::Data &data) override;
  uint64_t numBlocks() const override;
  uint64_t estimateNumFreeBytes() const override;
  uint64_t blockSizeFromPhysicalBlockSize(uint64_t blockSize) const override;
  void forEachBlock(std::function<void (const BlockId &)> callback) const override;

private:
  boost::filesystem::path _rootDir;

  static const std::string FORMAT_VERSION_HEADER_PREFIX;
  static const std::string FORMAT_VERSION_HEADER;

  boost::filesystem::path _getFilepath(const BlockId &blockId) const;
  static cpputils::Data _checkAndRemoveHeader(const cpputils::Data &data);
  static bool _isAcceptedCryfsHeader(const cpputils::Data &data);
  static bool _isOtherCryfsHeader(const cpputils::Data &data);
  static unsigned int formatVersionHeaderSize();

  DISALLOW_COPY_AND_ASSIGN(OnDiskBlockStore2);
};

}
}

#endif
