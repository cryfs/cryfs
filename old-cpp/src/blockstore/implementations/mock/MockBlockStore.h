#pragma once
#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_MOCK_MOCKBLOCKSTORE_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_MOCK_MOCKBLOCKSTORE_H_

#include <blockstore/implementations/testfake/FakeBlockStore.h>
#include <mutex>
#include "MockBlock.h"

namespace blockstore {
    namespace mock {

        /**
         * This is a blockstore that counts the number of loaded, resized, written, ... blocks.
         * It is used for testing that operations only access few blocks (performance tests).
         */
        class MockBlockStore final : public BlockStore {
        public:
            MockBlockStore(cpputils::unique_ref<BlockStore> baseBlockStore = cpputils::make_unique_ref<testfake::FakeBlockStore>())
                    : _mutex(), _baseBlockStore(std::move(baseBlockStore)), _loadedBlocks(), _createdBlocks(0), _writtenBlocks(), _resizedBlocks(), _removedBlocks() {
            }

            BlockId createBlockId() override {
                return _baseBlockStore->createBlockId();
            }

            boost::optional<cpputils::unique_ref<Block>> tryCreate(const BlockId &blockId, cpputils::Data data) override {
                _increaseNumCreatedBlocks();
                auto base = _baseBlockStore->tryCreate(blockId, std::move(data));
                if (base == boost::none) {
                    return boost::none;
                }
                return boost::optional<cpputils::unique_ref<Block>>(cpputils::make_unique_ref<MockBlock>(std::move(*base), this));
            }

            boost::optional<cpputils::unique_ref<Block>> load(const BlockId &blockId) override {
                _increaseNumLoadedBlocks(blockId);
                auto base = _baseBlockStore->load(blockId);
                if (base == boost::none) {
                    return boost::none;
                }
                return boost::optional<cpputils::unique_ref<Block>>(cpputils::make_unique_ref<MockBlock>(std::move(*base), this));
            }

            cpputils::unique_ref<Block> overwrite(const BlockId &blockId, cpputils::Data data) override {
                _increaseNumWrittenBlocks(blockId);
                return _baseBlockStore->overwrite(blockId, std::move(data));
            }

            void remove(const BlockId &blockId) override {
                _increaseNumRemovedBlocks(blockId);
                return _baseBlockStore->remove(blockId);
            }

            uint64_t numBlocks() const override {
                return _baseBlockStore->numBlocks();
            }

            uint64_t estimateNumFreeBytes() const override {
                return _baseBlockStore->estimateNumFreeBytes();
            }

            uint64_t blockSizeFromPhysicalBlockSize(uint64_t blockSize) const override {
                return _baseBlockStore->blockSizeFromPhysicalBlockSize(blockSize);
            }

            void forEachBlock(std::function<void(const BlockId &)> callback) const override {
                return _baseBlockStore->forEachBlock(callback);
            }

            void remove(cpputils::unique_ref<Block> block) override {
                _increaseNumRemovedBlocks(block->blockId());
                auto mockBlock = cpputils::dynamic_pointer_move<MockBlock>(block);
                ASSERT(mockBlock != boost::none, "Wrong block type");
                return _baseBlockStore->remove((*mockBlock)->releaseBaseBlock());
            }

            void flushBlock(Block* block) override {
                MockBlock* mockBlock = dynamic_cast<MockBlock*>(block);
                ASSERT(mockBlock != nullptr, "flushBlock got a block from the wrong block store");
                return _baseBlockStore->flushBlock(&*(mockBlock->_baseBlock));
            }

            void resetCounters() {
                _loadedBlocks = {};
                _createdBlocks = 0;
                _removedBlocks = {};
                _resizedBlocks = {};
                _writtenBlocks = {};
            }

            uint64_t createdBlocks() const {
                return _createdBlocks;
            }

            const std::vector<BlockId> &loadedBlocks() const {
                return _loadedBlocks;
            }

            const std::vector<BlockId> &removedBlocks() const {
                return _removedBlocks;
            }

            const std::vector<BlockId> &resizedBlocks() const {
                return _resizedBlocks;
            }

            const std::vector<BlockId> &writtenBlocks() const {
                return _writtenBlocks;
            }

            std::vector<BlockId> distinctWrittenBlocks() const {
                std::vector<BlockId> result(_writtenBlocks);
                std::sort(result.begin(), result.end(), [](const BlockId &lhs, const BlockId &rhs) {
                    return std::memcmp(lhs.data().data(), rhs.data().data(), lhs.BINARY_LENGTH) < 0;
                });
                result.erase(std::unique(result.begin(), result.end() ), result.end());
                return result;
            }

        private:
            void _increaseNumCreatedBlocks() {
                std::unique_lock<std::mutex> lock(_mutex);
                _createdBlocks += 1;
            }

            void _increaseNumLoadedBlocks(const BlockId &blockId) {
                std::unique_lock<std::mutex> lock(_mutex);
                _loadedBlocks.push_back(blockId);
            }

            void _increaseNumRemovedBlocks(const BlockId &blockId) {
                std::unique_lock<std::mutex> lock(_mutex);
                _removedBlocks.push_back(blockId);
            }

            void _increaseNumResizedBlocks(const BlockId &blockId) {
                std::unique_lock<std::mutex> lock(_mutex);
                _resizedBlocks.push_back(blockId);
            }

            void _increaseNumWrittenBlocks(const BlockId &blockId) {
                std::unique_lock<std::mutex> lock(_mutex);
                _writtenBlocks.push_back(blockId);
            }

            friend class MockBlock;

            std::mutex _mutex;
            cpputils::unique_ref<BlockStore> _baseBlockStore;

            std::vector<BlockId> _loadedBlocks;
            uint64_t _createdBlocks;
            std::vector<BlockId> _writtenBlocks;
            std::vector<BlockId> _resizedBlocks;
            std::vector<BlockId> _removedBlocks;

            DISALLOW_COPY_AND_ASSIGN(MockBlockStore);
        };

    }
}

#endif
