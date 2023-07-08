#include <unordered_set>
#include "LowToHighLevelBlockStore.h"
#include "LowToHighLevelBlock.h"

using cpputils::unique_ref;
using cpputils::Data;
using boost::none;
using boost::optional;
using std::string;

namespace blockstore {
namespace lowtohighlevel {

LowToHighLevelBlockStore::LowToHighLevelBlockStore(unique_ref<BlockStore2> baseBlockStore)
    : _baseBlockStore(std::move(baseBlockStore)) {
}

BlockId LowToHighLevelBlockStore::createBlockId() {
    // TODO Is this the right way?
    return BlockId::Random();
}

optional<unique_ref<Block>> LowToHighLevelBlockStore::tryCreate(const BlockId &blockId, Data data) {
    //TODO Easier implementation? This is only so complicated because of the cast LowToHighLevelBlock -> Block
    auto result = LowToHighLevelBlock::TryCreateNew(_baseBlockStore.get(), blockId, std::move(data));
    if (result == none) {
        return none;
    }
    return unique_ref<Block>(std::move(*result));
}

unique_ref<Block> LowToHighLevelBlockStore::overwrite(const BlockId &blockId, Data data) {
    return unique_ref<Block>(
        LowToHighLevelBlock::Overwrite(_baseBlockStore.get(), blockId, std::move(data))
    );
}

optional<unique_ref<Block>> LowToHighLevelBlockStore::load(const BlockId &blockId) {
    auto result = optional<unique_ref<Block>>(LowToHighLevelBlock::Load(_baseBlockStore.get(), blockId));
    if (result == none) {
      return none;
    }
    return unique_ref<Block>(std::move(*result));
}

void LowToHighLevelBlockStore::remove(const BlockId &blockId) {
    const bool success = _baseBlockStore->remove(blockId);
    if (!success) {
        throw std::runtime_error("Couldn't delete block with id " + blockId.ToString());
    }
}

uint64_t LowToHighLevelBlockStore::numBlocks() const {
    return _baseBlockStore->numBlocks();
}

uint64_t LowToHighLevelBlockStore::estimateNumFreeBytes() const {
    return _baseBlockStore->estimateNumFreeBytes();
}

uint64_t LowToHighLevelBlockStore::blockSizeFromPhysicalBlockSize(uint64_t blockSize) const {
    return _baseBlockStore->blockSizeFromPhysicalBlockSize(blockSize);
}

void LowToHighLevelBlockStore::forEachBlock(std::function<void (const BlockId &)> callback) const {
    _baseBlockStore->forEachBlock(std::move(callback));
}

}
}
