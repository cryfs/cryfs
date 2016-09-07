#include "BaseBlockWrapper.h"
#include "CachingBlockStore.h"

using cpputils::unique_ref;
using cpputils::Data;
using cpputils::make_left;
using cpputils::make_right;
using cpputils::either;
using std::unique_lock;
using std::mutex;
using boost::none;

namespace blockstore {
    namespace caching {

        BaseBlockWrapper::BaseBlockWrapper(unique_ref<Block> baseBlock, CachingBlockStore *cachingBlockStore)
            : _cachingBlockStore(cachingBlockStore),
              _baseBlock(make_right<NotLoadedBlock, unique_ref<Block>>(std::move(baseBlock))),
              _isValid(true),
              _mutex() {
        }

        BaseBlockWrapper::BaseBlockWrapper(const Key &key, size_t size, CachingBlockStore *cachingBlockStore)
            : _cachingBlockStore(cachingBlockStore),
              _baseBlock(make_left<NotLoadedBlock, unique_ref<Block>>(NotLoadedBlock(key, size))),
              _isValid(true),
              _mutex() {
        }

        BaseBlockWrapper::BaseBlockWrapper(BaseBlockWrapper &&rhs)
            : _cachingBlockStore(std::move(rhs._cachingBlockStore)),
              _baseBlock(rhs._releaseBaseBlock()),
              _isValid(true),
              _mutex() {
            //rhs._isValid = false; // This is already set in rhs._releaseBaseBlock()
        }

        BaseBlockWrapper::~BaseBlockWrapper() {
            if (_isValid) {
                flush();
            }
        }

        either<NotLoadedBlock, unique_ref<Block>> BaseBlockWrapper::_releaseBaseBlock() {
            unique_lock<mutex> lock(_mutex);
            ASSERT(_isValid, "Trying to release base block from an invalid instance");
            _isValid = false;
            return std::move(_baseBlock);
        };

        bool BaseBlockWrapper::isValid() const {
            return _isValid;
        }

        const void *BaseBlockWrapper::data() const {
            _ensureIsFullyLoaded();
            return _baseBlock.right()->data();
        }

        void BaseBlockWrapper::_ensureIsFullyLoaded() const {
            unique_lock<mutex> lock(_mutex);
            if (_baseBlock.is_left()) {
                _loadBaseBlock();
            }
        }

        void BaseBlockWrapper::_loadBaseBlock() const {
            ASSERT(_baseBlock.is_left(), "Block already loaded");
            NotLoadedBlock notLoadedBlock = std::move(_baseBlock.left());
            _cachingBlockStore->unregisterBlockThatMightNotBeInTheBaseStore(notLoadedBlock.key);
            if (notLoadedBlock.validRegion.isCovered(0, notLoadedBlock.data.size())) {
                auto baseBlock = _baseBlockStore()->overwrite(notLoadedBlock.key, std::move(notLoadedBlock.data));
                _baseBlock = std::move(baseBlock);
            } else {
                _baseBlock = _baseBlockStore()->loadOrCreate(notLoadedBlock.key, notLoadedBlock.data.size());
                ASSERT(_baseBlock.right()->size() == notLoadedBlock.data.size(), "LoadOrCreate should resize the block");
                notLoadedBlock.validRegion.forEachInterval([this, &notLoadedBlock] (size_t begin, size_t end) {
                    _baseBlock.right()->write(notLoadedBlock.data.dataOffset(begin), begin, end-begin);
                });
            }
        }

        void BaseBlockWrapper::write(const void *source, uint64_t offset, uint64_t size) {
            unique_lock<mutex> lock(_mutex);
            if (_baseBlock.is_right()) {
                _baseBlock.right()->write(source, offset, size);
            } else {
                ASSERT(offset <= _baseBlock.left().data.size() && offset+size <= _baseBlock.left().data.size(), "Write out of bounds");
                std::memcpy(_baseBlock.left().data.dataOffset(offset), source, size);
                _baseBlock.left().validRegion.add(offset, offset+size);
            }
        }

        void BaseBlockWrapper::flush() {
            _ensureIsFullyLoaded();
            _baseBlock.right()->flush();
        }

        size_t BaseBlockWrapper::size() const {
            unique_lock<mutex> lock(_mutex);
            if (_baseBlock.is_right()) {
                return _baseBlock.right()->size();
            } else {
                return _baseBlock.left().data.size();
            }
        }

        const Key &BaseBlockWrapper::key() const {
            unique_lock<mutex> lock(_mutex);
            if (_baseBlock.is_right()) {
                return _baseBlock.right()->key();
            } else {
                return _baseBlock.left().key;
            }
        }

        void BaseBlockWrapper::remove() {
            unique_lock<mutex> lock(_mutex);
            if (_baseBlock.is_right()) {
                _baseBlockStore()->remove(std::move(_baseBlock.right()));
            } else {
                _cachingBlockStore->unregisterBlockThatMightNotBeInTheBaseStore(_baseBlock.left().key);
                _baseBlockStore()->removeIfExists(_baseBlock.left().key);
            }
            _isValid = false;
        }

        void BaseBlockWrapper::resize(size_t newSize) {
            unique_lock<mutex> lock(_mutex);
            if (_baseBlock.is_right()) {
                _baseBlock.right()->resize(newSize);
            } else {
                Data newData(newSize);
                std::memcpy(newData.data(), _baseBlock.left().data.data(), std::min(_baseBlock.left().data.size(), newSize));
                _baseBlock.left().data = std::move(newData);
            }
        }

        BlockStore *BaseBlockWrapper::_baseBlockStore() const {
            return _cachingBlockStore->baseBlockStore();
        }

    }
}
