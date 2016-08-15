#include "CachingBlockStore.h"
#include "CachedBlock.h"

using boost::optional;
using boost::none;
using cpputils::unique_ref;
using cpputils::make_unique_ref;
using cpputils::dynamic_pointer_move;
using cpputils::Data;
using std::function;

namespace blockstore{
    namespace caching {

        CachingBlockStore::CachingBlockStore(cpputils::unique_ref<BlockStore> baseBlockStore)
            :_baseBlockStore(std::move(baseBlockStore)), _cache(), _blocksThatMightNotBeInTheBaseStore() {
        }

        CachingBlockStore::~CachingBlockStore() {
            _cache.flush();
            ASSERT(_blocksThatMightNotBeInTheBaseStore.size() == 0, "A block wrapper that was created with either tryCreate() or loadOrCreate() didn't deregister itself");
        }

        void CachingBlockStore::returnToCache(BaseBlockWrapper baseBlock) {
            Key key = baseBlock.key();
            _cache.push(key, std::move(baseBlock));
        }

        Key CachingBlockStore::createKey() {
            return _baseBlockStore->createKey();
        }

        optional<unique_ref<Block>> CachingBlockStore::tryCreate(const Key &key, Data data) {
            auto baseBlock = _tryCreateBaseBlockWrapper(key, data.size());
            if (baseBlock == none) {
                return none;
            }
            baseBlock->write(data.data(), 0, data.size());
            return optional<unique_ref<Block>>(make_unique_ref<CachedBlock>(std::move(*baseBlock), this));
        }

        unique_ref<Block> CachingBlockStore::overwrite(const Key &key, Data data) {
            auto created = loadOrCreate(key, data.size());
            created->write(data.data(), 0, data.size());
            return created;
        }

        optional<unique_ref<Block>> CachingBlockStore::load(const Key &key) {
            auto baseBlock = _loadBaseBlockWrapper(key);
            if (baseBlock == none) {
                return none;
            }
            return optional<unique_ref<Block>>(make_unique_ref<CachedBlock>(std::move(*baseBlock), this));
        }

        optional<BaseBlockWrapper> CachingBlockStore::_loadBaseBlockWrapper(const Key &key) {
            auto fromCache = _cache.pop(key);
            if (fromCache != none) {
                return std::move(*fromCache);
            }
            auto fromBaseStore = _baseBlockStore->load(key);
            if (fromBaseStore != none) {
                return BaseBlockWrapper(std::move(*fromBaseStore), this);
            }
            return none;
        }

        unique_ref<Block> CachingBlockStore::loadOrCreate(const Key &key, size_t size) {
            auto baseBlock = _loadOrCreateBaseBlockWrapper(key, size);
            return make_unique_ref<CachedBlock>(std::move(baseBlock), this);
        }

        BaseBlockWrapper CachingBlockStore::_loadOrCreateBaseBlockWrapper(const Key &key, size_t size) {
            auto fromCache = _cache.pop(key);
            if (fromCache != none) {
                if (size != fromCache->size()) {
                    fromCache->resize(size);
                }
                return std::move(*fromCache);
            }
            std::unique_lock<std::mutex> lock(_blocksThatMightNotBeInTheBaseStoreMutex);
            _blocksThatMightNotBeInTheBaseStore.insert(key);
            return BaseBlockWrapper(key, size, this);
        }

        optional<BaseBlockWrapper> CachingBlockStore::_tryCreateBaseBlockWrapper(const Key &key, size_t size) {
            auto fromCache = _cache.pop(key);
            if (fromCache != none) {
                return none; // Block exists already
            }
            if (_baseBlockStore->exists(key)) {
                return none;
            }
            std::unique_lock<std::mutex> lock(_blocksThatMightNotBeInTheBaseStoreMutex);
            _blocksThatMightNotBeInTheBaseStore.insert(key);
            return BaseBlockWrapper(key, size, this);
        }

        void CachingBlockStore::remove(const Key &key) {
            auto fromCache = _cache.pop(key);
            if (fromCache != none) {
                fromCache->remove();
            } else {
                _baseBlockStore->remove(key);
            }
        }

        void CachingBlockStore::remove(unique_ref<Block> block) {
            auto cachedBlock = dynamic_pointer_move<CachedBlock>(block);
            ASSERT(cachedBlock != none, "Given block is not a CachedBlock");
            (*cachedBlock)->releaseBaseBlockWrapper().remove();
        }

        uint64_t CachingBlockStore::numBlocks() const {
            // TODO How often is numBlocks called? Is this imperformant implementation a problem? It's not that simple, because some blocks in the cache already exist in the base store and some don't.
            uint64_t num = 0;
            forEachBlock([&num] (const Key &) {
                num += 1;
            });
            return num;
        }

        uint64_t CachingBlockStore::estimateNumFreeBytes() const {
            return _baseBlockStore->estimateNumFreeBytes();
        }

        uint64_t CachingBlockStore::blockSizeFromPhysicalBlockSize(uint64_t blockSize) const {
            return _baseBlockStore->blockSizeFromPhysicalBlockSize(blockSize);
        }

        void CachingBlockStore::forEachBlock(function<void (const Key &)> callback) const {
            std::unique_lock<std::mutex> lock(_blocksThatMightNotBeInTheBaseStoreMutex);
            std::unordered_set<blockstore::Key> blocksThatMightNotBeInTheBaseStore = _blocksThatMightNotBeInTheBaseStore;
            lock.unlock();

            _baseBlockStore->forEachBlock([&blocksThatMightNotBeInTheBaseStore, &callback] (const Key &key) {
                blocksThatMightNotBeInTheBaseStore.erase(key);
                callback(key);
            });
            // Also call the callback for blocks that are only in the cache, not in the base store.
            for (const auto &block : blocksThatMightNotBeInTheBaseStore) {
                callback(block);
            }
        }

        bool CachingBlockStore::exists(const Key &key) const {
            std::unique_lock<std::mutex> lock(_blocksThatMightNotBeInTheBaseStoreMutex);
            return _blocksThatMightNotBeInTheBaseStore.count(key) || _baseBlockStore->exists(key);
        }

        void CachingBlockStore::unregisterBlockThatMightNotBeInTheBaseStore(const Key &key) {
            std::unique_lock<std::mutex> lock(_blocksThatMightNotBeInTheBaseStoreMutex);
            _blocksThatMightNotBeInTheBaseStore.erase(key);
        }

        BlockStore *CachingBlockStore::baseBlockStore() {
            return _baseBlockStore.get();
        }

    }
}
