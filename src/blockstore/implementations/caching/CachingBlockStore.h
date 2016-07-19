#pragma once
#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_CACHING_CACHINGBLOCKSTORE_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_CACHING_CACHINGBLOCKSTORE_H_

#include <cpp-utils/macros.h>
#include <blockstore/interface/BlockStore.h>
#include "cache/Cache.h"
#include "BaseBlockWrapper.h"
#include <unordered_set>

namespace blockstore {
    namespace caching {

        //TODO Mutexes/Locking, here and also in used classes like BaseBlockWrapper

        class CachingBlockStore final : public BlockStore {
        public:
            explicit CachingBlockStore(cpputils::unique_ref<BlockStore> baseBlockStore);
            ~CachingBlockStore();

            Key createKey() override;
            boost::optional<cpputils::unique_ref<Block>> tryCreate(const Key &key, cpputils::Data data) override;
            cpputils::unique_ref<Block> overwrite(const Key &key, cpputils::Data data) override;
            boost::optional<cpputils::unique_ref<Block>> load(const Key &key) override;
            cpputils::unique_ref<Block> loadOrCreate(const Key &key, size_t size) override;
            void remove(const Key &key) override;
            void remove(cpputils::unique_ref<Block> block) override;
            uint64_t numBlocks() const override;
            uint64_t estimateNumFreeBytes() const override;
            uint64_t blockSizeFromPhysicalBlockSize(uint64_t blockSize) const override;
            void forEachBlock(std::function<void (const Key &)> callback) const override;
            bool exists(const Key &key) const override;

            void returnToCache(BaseBlockWrapper baseBlock);
            void unregisterBlockThatMightNotBeInTheBaseStore(const Key &key);
            BlockStore *baseBlockStore();

        private:
            cpputils::unique_ref<BlockStore> _baseBlockStore;
            Cache<Key, BaseBlockWrapper, 1000> _cache;
            std::unordered_set<Key> _blocksThatMightNotBeInTheBaseStore;
            mutable std::mutex _blocksThatMightNotBeInTheBaseStoreMutex;

            boost::optional<BaseBlockWrapper> _loadBaseBlockWrapper(const Key &key);
            BaseBlockWrapper _loadOrCreateBaseBlockWrapper(const Key &key, size_t size);
            boost::optional<BaseBlockWrapper> _tryCreateBaseBlockWrapper(const Key &key, size_t size);

            DISALLOW_COPY_AND_ASSIGN(CachingBlockStore);
        };

    }
}


#endif
