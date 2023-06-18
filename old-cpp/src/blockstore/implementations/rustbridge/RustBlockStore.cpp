#include "RustBlockStore.h"
#include "RustBlock.h"
#include "helpers.h"

namespace blockstore
{
    namespace rust
    {
        namespace {
            cpputils::unique_ref<Block> cast_block(::rust::Box<::blockstore::rust::bridge::RustBlockBridge> block)
            {
                cpputils::unique_ref<Block> result = cpputils::make_unique_ref<RustBlock>(std::move(block));
                return result;
            }

            boost::optional<cpputils::unique_ref<Block>> cast_optional_block(::rust::Box<::blockstore::rust::bridge::OptionRustBlockBridge> optionBlock)
            {
                if (optionBlock->has_value())
                {
                    // TODO defer to cast_block
                    auto data = optionBlock->extract_value();
                    cpputils::unique_ref<Block> result = cpputils::make_unique_ref<RustBlock>(std::move(data));
                    return result;
                }
                else
                {
                    return boost::none;
                }
            }
        }

        RustBlockStore::RustBlockStore(::rust::Box<bridge::RustBlockStoreBridge> blockStore)
        : _blockStore(std::move(blockStore)) {}

        RustBlockStore::~RustBlockStore() {
            _blockStore->async_drop();
        }

        BlockId RustBlockStore::createBlockId() {
            return helpers::cast_blockid(*_blockStore->create_block_id());
        }

        boost::optional<cpputils::unique_ref<Block>> RustBlockStore::tryCreate(const BlockId &blockId, cpputils::Data data) {
            return cast_optional_block(_blockStore->try_create(*helpers::cast_blockid(blockId), helpers::cast_data(data)));
        }

        boost::optional<cpputils::unique_ref<Block>> RustBlockStore::load(const BlockId &blockId) {
            return cast_optional_block(_blockStore->load(*helpers::cast_blockid(blockId)));
        }

        cpputils::unique_ref<Block> RustBlockStore::overwrite(const blockstore::BlockId &blockId, cpputils::Data data) {
            return cast_block(_blockStore->overwrite(*helpers::cast_blockid(blockId), helpers::cast_data(std::move(data))));
        }

        void RustBlockStore::remove(const BlockId &blockId) {
            _blockStore->remove(*helpers::cast_blockid(blockId));
        }

        uint64_t RustBlockStore::numBlocks() const {
            return _blockStore->num_blocks();
        }

        uint64_t RustBlockStore::estimateNumFreeBytes() const {
            return _blockStore->estimate_num_free_bytes();
        }

        uint64_t RustBlockStore::blockSizeFromPhysicalBlockSize(uint64_t blockSize) const {
            return _blockStore->block_size_from_physical_block_size(blockSize);
        }

        void RustBlockStore::forEachBlock(std::function<void (const BlockId &)> callback) const {
            auto blocks = _blockStore->all_blocks();
            for (const auto &block : blocks)
            {
                callback(helpers::cast_blockid(block));
            }
        }

        void RustBlockStore::flushBlock(Block* block) {
            RustBlock* rustBlock = dynamic_cast<RustBlock*>(block);
            ASSERT(rustBlock != nullptr, "flushBlock got a block from the wrong block store");
            _blockStore->flush_block(rustBlock->_block);
        }
    }
}
