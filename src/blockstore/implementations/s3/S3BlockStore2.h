#pragma once
#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_S3_S3BLOCKSTORE2_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_S3_S3BLOCKSTORE2_H_

#include "../../interface/BlockStore2.h"
#include <cpp-utils/macros.h>
#include <unordered_map>

namespace blockstore {
namespace s3 {

class S3BlockStore2 final: public BlockStore2 {
public:
  S3BlockStore2();
  ~S3BlockStore2();

  bool tryCreate(const BlockId &blockId, const cpputils::Data &data) override;
  bool remove(const BlockId &blockId) override;
  boost::optional<cpputils::Data> load(const BlockId &blockId) const override;
  void store(const BlockId &blockId, const cpputils::Data &data) override;
  uint64_t numBlocks() const override;
  uint64_t estimateNumFreeBytes() const override;
  uint64_t blockSizeFromPhysicalBlockSize(uint64_t blockSize) const override;
  void forEachBlock(std::function<void (const BlockId &)> callback) const override;

private:
  class AwsS3SDK;
  cpputils::unique_ref<AwsS3SDK> _sdk;

  DISALLOW_COPY_AND_ASSIGN(S3BlockStore2);
};

}
}

#endif
