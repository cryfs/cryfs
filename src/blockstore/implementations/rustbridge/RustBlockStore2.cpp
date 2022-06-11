#include "RustBlockStore2.h"

namespace blockstore {
namespace rust {

namespace {
bridge::BlockId cast_blockid(const BlockId &blockId) {
    bridge::BlockId result;
    blockId.ToBinary(result.id.data());
    return result;
}
BlockId cast_blockid(const bridge::BlockId& blockId) {
    return BlockId::FromBinary(blockId.id.data());
}
::rust::Slice<const uint8_t> cast_data(const cpputils::Data& data) {
    return ::rust::Slice<const uint8_t> {static_cast<const uint8_t*>(data.data()), data.size()};
}
boost::optional<cpputils::Data> cast_optional_data(const ::blockstore::rust::bridge::OptionData* optionData) {
    if (optionData->has_value()) {
        auto data = optionData->value();
        cpputils::Data result(data.size());
        std::memcpy(result.data(), data.data(), data.size());
        return result;
    } else {
        return boost::none;
    }
}
}

RustBlockStore2::RustBlockStore2(::rust::Box<bridge::RustBlockStore2Bridge> blockStore)
: _blockStore(std::move(blockStore)) {
}

bool RustBlockStore2::tryCreate(const BlockId &blockId, const cpputils::Data &data) {
    return _blockStore->try_create(cast_blockid(blockId), cast_data(data));
}

bool RustBlockStore2::remove(const BlockId &blockId) {
    return _blockStore->remove(cast_blockid(blockId));
}

boost::optional<cpputils::Data> RustBlockStore2::load(const BlockId &blockId) const {
    return cast_optional_data(&*_blockStore->load(cast_blockid(blockId)));
}

void RustBlockStore2::store(const BlockId &blockId, const cpputils::Data &data) {
    return _blockStore->store(cast_blockid(blockId), cast_data(data));
}

uint64_t RustBlockStore2::numBlocks() const {
    return _blockStore->num_blocks();
}

uint64_t RustBlockStore2::estimateNumFreeBytes() const {
    return _blockStore->estimate_num_free_bytes();
}

uint64_t RustBlockStore2::blockSizeFromPhysicalBlockSize(uint64_t blockSize) const {
    return _blockStore->block_size_from_physical_block_size(blockSize);
}

void RustBlockStore2::forEachBlock(std::function<void (const BlockId &)> callback) const {
    auto blocks = _blockStore->all_blocks();
    for (const auto& block : blocks) {
        callback(cast_blockid(block));
    }
}

}
}
