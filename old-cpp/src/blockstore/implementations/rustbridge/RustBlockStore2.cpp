#include "RustBlockStore2.h"
#include "helpers.h"

namespace blockstore
{
    namespace rust
    {
        RustBlockStore2::RustBlockStore2(::rust::Box<bridge::RustBlockStore2Bridge> blockStore)
            : _blockStore(std::move(blockStore))
        {
        }

        RustBlockStore2::~RustBlockStore2() {
            _blockStore->async_drop();
        }

        bool RustBlockStore2::tryCreate(const BlockId &blockId, const cpputils::Data &data)
        {
            return _blockStore->try_create(*helpers::cast_blockid(blockId), helpers::cast_data(data));
        }

        bool RustBlockStore2::remove(const BlockId &blockId)
        {
            return _blockStore->remove(*helpers::cast_blockid(blockId));
        }

        boost::optional<cpputils::Data> RustBlockStore2::load(const BlockId &blockId) const
        {
            return helpers::cast_optional_data(&*_blockStore->load(*helpers::cast_blockid(blockId)));
        }

        void RustBlockStore2::store(const BlockId &blockId, const cpputils::Data &data)
        {
            return _blockStore->store(*helpers::cast_blockid(blockId), helpers::cast_data(data));
        }

        uint64_t RustBlockStore2::numBlocks() const
        {
            return _blockStore->num_blocks();
        }

        uint64_t RustBlockStore2::estimateNumFreeBytes() const
        {
            return _blockStore->estimate_num_free_bytes();
        }

        uint64_t RustBlockStore2::blockSizeFromPhysicalBlockSize(uint64_t blockSize) const
        {
            return _blockStore->block_size_from_physical_block_size(blockSize);
        }

        void RustBlockStore2::forEachBlock(std::function<void(const BlockId &)> callback) const
        {
            auto blocks = _blockStore->all_blocks();
            for (const auto &block : blocks)
            {
                callback(helpers::cast_blockid(block));
            }
        }

    }
}
