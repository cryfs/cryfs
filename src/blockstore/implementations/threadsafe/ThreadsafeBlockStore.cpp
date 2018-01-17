#include <unordered_set>
#include "ThreadsafeBlockStore.h"
#include "ThreadsafeBlock.h"

using cpputils::unique_ref;
using cpputils::make_unique_ref;
using cpputils::Data;
using boost::none;
using boost::optional;
using std::string;
using cpputils::MutexPoolLock;
using std::unique_lock;
using std::mutex;

namespace blockstore {
namespace threadsafe {

ThreadsafeBlockStore::ThreadsafeBlockStore(unique_ref<BlockStore> baseBlockStore)
    : _baseBlockStore(std::move(baseBlockStore)), _checkedOutBlocks(), _structureMutex() {
}

BlockId ThreadsafeBlockStore::createBlockId() {
    return _baseBlockStore->createBlockId();
}

optional<unique_ref<Block>> ThreadsafeBlockStore::tryCreate(const BlockId &blockId, Data data) {
    unique_lock<mutex> structureLock(_structureMutex);
    MutexPoolLock<BlockId> lock(&_checkedOutBlocks, blockId);
    auto created = _baseBlockStore->tryCreate(blockId, std::move(data));
    structureLock.unlock();

    if (created == none) {
        return none;
    }
    return unique_ref<Block>(make_unique_ref<ThreadsafeBlock>(std::move(*created), std::move(lock)));
}

unique_ref<Block> ThreadsafeBlockStore::overwrite(const BlockId &blockId, Data data) {
    unique_lock<mutex> structureLock(_structureMutex);
    MutexPoolLock<BlockId> lock(&_checkedOutBlocks, blockId);
    auto overwritten = _baseBlockStore->overwrite(blockId, std::move(data));
    structureLock.unlock();

    return make_unique_ref<ThreadsafeBlock>(std::move(overwritten), std::move(lock));
}

optional<unique_ref<Block>> ThreadsafeBlockStore::load(const BlockId &blockId) {
    MutexPoolLock<BlockId> lock(&_checkedOutBlocks, blockId);
    auto loaded = _baseBlockStore->load(blockId);
    if (loaded == none) {
      return none;
    }
    return unique_ref<Block>(make_unique_ref<ThreadsafeBlock>(std::move(*loaded), std::move(lock)));
}

void ThreadsafeBlockStore::remove(const BlockId &blockId) {
    unique_lock<mutex> structureLock(_structureMutex);
    MutexPoolLock<BlockId> lock(&_checkedOutBlocks, blockId);
    _baseBlockStore->remove(blockId);
}

uint64_t ThreadsafeBlockStore::numBlocks() const {
    unique_lock<mutex> structureLock(_structureMutex);
    return _baseBlockStore->numBlocks();
}

uint64_t ThreadsafeBlockStore::estimateNumFreeBytes() const {
    unique_lock<mutex> structureLock(_structureMutex);
    return _baseBlockStore->estimateNumFreeBytes();
}

uint64_t ThreadsafeBlockStore::blockSizeFromPhysicalBlockSize(uint64_t blockSize) const {
    return _baseBlockStore->blockSizeFromPhysicalBlockSize(blockSize);
}

void ThreadsafeBlockStore::forEachBlock(std::function<void (const BlockId &)> callback) const {
    unique_lock<mutex> structureLock(_structureMutex);
    _baseBlockStore->forEachBlock(std::move(callback));
}

}
}
