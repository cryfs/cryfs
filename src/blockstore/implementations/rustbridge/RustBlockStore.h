#pragma once
#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_RUSTBRIDGE_RUSTBLOCKSTORE_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_RUSTBRIDGE_RUSTBLOCKSTORE_H_

#include "../../interface/BlockStore.h"
#include <cpp-utils/macros.h>
#include <unordered_map>
#include "cxxbridge/cryfs-blockstore/src/blockstore/cppbridge.rs.h"

namespace blockstore
{
  namespace rust
  {

    class RustBlockStore final : public BlockStore
    {
    public:
        RustBlockStore(::rust::Box<bridge::RustBlockStoreBridge> blockStore);

        BlockId createBlockId() override;
        boost::optional<cpputils::unique_ref<Block>> tryCreate(const BlockId &blockId, cpputils::Data data) override;
        boost::optional<cpputils::unique_ref<Block>> load(const BlockId &blockId) override;
        cpputils::unique_ref<Block> overwrite(const blockstore::BlockId &blockId, cpputils::Data data) override;
        void remove(const BlockId &blockId) override;
        uint64_t numBlocks() const override;
        uint64_t estimateNumFreeBytes() const override;
        uint64_t blockSizeFromPhysicalBlockSize(uint64_t blockSize) const override;
        void forEachBlock(std::function<void (const BlockId &)> callback) const override;

    private:
      ::rust::Box<bridge::RustBlockStoreBridge> _blockStore;

      DISALLOW_COPY_AND_ASSIGN(RustBlockStore);
    };

  } // namespace rust
} // namespace blockstore

#endif
