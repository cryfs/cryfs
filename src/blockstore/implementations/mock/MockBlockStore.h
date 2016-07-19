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

            Key createKey() override {
                return _baseBlockStore->createKey();
            }

            boost::optional<cpputils::unique_ref<Block>> tryCreate(const Key &key, cpputils::Data data) override {
                _increaseNumCreatedBlocks();
                auto base = _baseBlockStore->tryCreate(key, std::move(data));
                if (base == boost::none) {
                    return boost::none;
                }
                return boost::optional<cpputils::unique_ref<Block>>(cpputils::make_unique_ref<MockBlock>(std::move(*base), this));
            }

            boost::optional<cpputils::unique_ref<Block>> load(const Key &key) override {
                _increaseNumLoadedBlocks(key);
                auto base = _baseBlockStore->load(key);
                if (base == boost::none) {
                    return boost::none;
                }
                return boost::optional<cpputils::unique_ref<Block>>(cpputils::make_unique_ref<MockBlock>(std::move(*base), this));
            }

            cpputils::unique_ref<Block> overwrite(const Key &key, cpputils::Data data) override {
                _increaseNumWrittenBlocks(key);
                return _baseBlockStore->overwrite(key, std::move(data));
            }

            void remove(const Key &key) override {
                _increaseNumRemovedBlocks(key);
                return _baseBlockStore->remove(key);
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

            void forEachBlock(std::function<void(const Key &)> callback) const override {
                return _baseBlockStore->forEachBlock(callback);
            }

            void remove(cpputils::unique_ref<Block> block) override {
                _increaseNumRemovedBlocks(block->key());
                auto mockBlock = cpputils::dynamic_pointer_move<MockBlock>(block);
                ASSERT(mockBlock != boost::none, "Wrong block type");
                return _baseBlockStore->remove((*mockBlock)->releaseBaseBlock());
            }

            bool exists(const Key &key) const override {
                return _baseBlockStore->exists(key);
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

            const std::vector<Key> &loadedBlocks() const {
                return _loadedBlocks;
            }

            const std::vector<Key> &removedBlocks() const {
                return _removedBlocks;
            }

            const std::vector<Key> &resizedBlocks() const {
                return _resizedBlocks;
            }

            const std::vector<Key> &writtenBlocks() const {
                return _writtenBlocks;
            }

            std::vector<Key> distinctWrittenBlocks() const {
                std::vector<Key> result(_writtenBlocks);
                std::sort(result.begin(), result.end(), [](const Key &lhs, const Key &rhs) {
                    return std::memcmp(lhs.data(), rhs.data(), lhs.BINARY_LENGTH) < 0;
                });
                result.erase(std::unique(result.begin(), result.end() ), result.end());
                return result;
            }

        private:
            void _increaseNumCreatedBlocks() {
                std::unique_lock<std::mutex> lock(_mutex);
                _createdBlocks += 1;
            }

            void _increaseNumLoadedBlocks(const Key &key) {
                std::unique_lock<std::mutex> lock(_mutex);
                _loadedBlocks.push_back(key);
            }

            void _increaseNumRemovedBlocks(const Key &key) {
                std::unique_lock<std::mutex> lock(_mutex);
                _removedBlocks.push_back(key);
            }

            void _increaseNumResizedBlocks(const Key &key) {
                std::unique_lock<std::mutex> lock(_mutex);
                _resizedBlocks.push_back(key);
            }

            void _increaseNumWrittenBlocks(const Key &key) {
                std::unique_lock<std::mutex> lock(_mutex);
                _writtenBlocks.push_back(key);
            }

            friend class MockBlock;

            std::mutex _mutex;
            cpputils::unique_ref<BlockStore> _baseBlockStore;

            std::vector<Key> _loadedBlocks;
            uint64_t _createdBlocks;
            std::vector<Key> _writtenBlocks;
            std::vector<Key> _resizedBlocks;
            std::vector<Key> _removedBlocks;

            DISALLOW_COPY_AND_ASSIGN(MockBlockStore);
        };

    }
}

#endif
