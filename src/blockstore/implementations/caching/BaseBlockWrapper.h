#pragma once
#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_CACHING_BASEBLOCKWRAPPER_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_CACHING_BASEBLOCKWRAPPER_H_

#include <cpp-utils/macros.h>
#include <cpp-utils/pointer/unique_ref.h>
#include <cpp-utils/either.h>
#include "IntervalSet.h"
#include <blockstore/interface/Block.h>
#include <cpp-utils/data/Data.h>
#include <blockstore/interface/BlockStore.h>
#include <unordered_set>

namespace blockstore {
    namespace caching {
        class BaseBlockWrapper;
        class CachingBlockStore;

        struct NotLoadedBlock final {
            NotLoadedBlock(const Key &_key, size_t _size): key(_key), data(_size), validRegion() {}
            NotLoadedBlock(NotLoadedBlock &&rhs) = default;
            NotLoadedBlock &operator=(NotLoadedBlock &&rhs) = default;

            blockstore::Key key;
            cpputils::Data data;
            IntervalSet<size_t> validRegion;
        private:
            DISALLOW_COPY_AND_ASSIGN(NotLoadedBlock);
        };

        class BaseBlockWrapper final {
        public:
            BaseBlockWrapper(cpputils::unique_ref <Block> baseBlock, CachingBlockStore *cachingBlockStore);
            BaseBlockWrapper(const Key &key, size_t size, CachingBlockStore *cachingBlockStore);
            BaseBlockWrapper(BaseBlockWrapper &&rhs);

            ~BaseBlockWrapper();

            const Key &key() const;

            const void *data() const;

            void write(const void *source, uint64_t offset, uint64_t size);

            void flush();

            size_t size() const;

            void remove();

            void resize(size_t newSize);

            bool isValid() const;

        private:
            CachingBlockStore *_cachingBlockStore;
            mutable cpputils::either<NotLoadedBlock, cpputils::unique_ref<Block>> _baseBlock;
            bool _isValid;
            mutable std::mutex _mutex;

            cpputils::either<NotLoadedBlock, cpputils::unique_ref<Block>> _releaseBaseBlock();
            void _ensureIsFullyLoaded() const;
            void _loadBaseBlock() const;
            BlockStore *_baseBlockStore() const;

            DISALLOW_COPY_AND_ASSIGN(BaseBlockWrapper);
        };
    }
}


#endif
