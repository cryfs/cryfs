#pragma once
#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_RUSTBRIDGE_RUSTBLOCKSTORE2_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_RUSTBRIDGE_RUSTBLOCKSTORE2_H_

#include "../../interface/BlockStore2.h"
#include <cpp-utils/macros.h>
#include <unordered_map>
#include "cxxbridge/cryfs-cppbridge/src/blockstore.rs.h"

namespace blockstore
{
  namespace rust
  {

    class RustBlockStore2 final : public BlockStore2
    {
    public:
      RustBlockStore2(::rust::Box<bridge::RustBlockStore2Bridge> blockStore);
      ~RustBlockStore2();

      bool tryCreate(const BlockId &blockId, const cpputils::Data &data) override;
      bool remove(const BlockId &blockId) override;
      boost::optional<cpputils::Data> load(const BlockId &blockId) const override;
      void store(const BlockId &blockId, const cpputils::Data &data) override;
      uint64_t numBlocks() const override;
      uint64_t estimateNumFreeBytes() const override;
      uint64_t blockSizeFromPhysicalBlockSize(uint64_t blockSize) const override;
      void forEachBlock(std::function<void(const BlockId &)> callback) const override;

    private:
      ::rust::Box<bridge::RustBlockStore2Bridge> _blockStore;

      DISALLOW_COPY_AND_ASSIGN(RustBlockStore2);
    };

  } // namespace rust
} // namespace blockstore

#endif
