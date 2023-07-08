#pragma once
#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_COMPRESSING_COMPRESSINGBLOCKSTORE_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_COMPRESSING_COMPRESSINGBLOCKSTORE_H_

#include "../../interface/BlockStore.h"
#include "CompressedBlock.h"

namespace blockstore {
namespace compressing {

template<class Compressor>
class CompressingBlockStore final: public BlockStore {
public:
    CompressingBlockStore(cpputils::unique_ref<BlockStore> baseBlockStore);
    ~CompressingBlockStore() override;

    BlockId createBlockId() override;
    boost::optional<cpputils::unique_ref<Block>> tryCreate(const BlockId &blockId, cpputils::Data data) override;
    boost::optional<cpputils::unique_ref<Block>> load(const BlockId &blockId) override;
    cpputils::unique_ref<Block> overwrite(const blockstore::BlockId &blockId, cpputils::Data data) override;
    void remove(const BlockId &blockId) override;
    uint64_t numBlocks() const override;
    uint64_t estimateNumFreeBytes() const override;
    uint64_t blockSizeFromPhysicalBlockSize(uint64_t blockSize) const override;
    void forEachBlock(std::function<void (const BlockId &)> callback) const override;

private:
    cpputils::unique_ref<BlockStore> _baseBlockStore;

    DISALLOW_COPY_AND_ASSIGN(CompressingBlockStore);
};

template<class Compressor>
CompressingBlockStore<Compressor>::CompressingBlockStore(cpputils::unique_ref<BlockStore> baseBlockStore)
        : _baseBlockStore(std::move(baseBlockStore)) {
}

template<class Compressor>
CompressingBlockStore<Compressor>::~CompressingBlockStore() {
}

template<class Compressor>
BlockId CompressingBlockStore<Compressor>::createBlockId() {
    return _baseBlockStore->createBlockId();
}

template<class Compressor>
boost::optional<cpputils::unique_ref<Block>> CompressingBlockStore<Compressor>::tryCreate(const BlockId &blockId, cpputils::Data data) {
    auto result = CompressedBlock<Compressor>::TryCreateNew(_baseBlockStore.get(), blockId, std::move(data));
    if (result == boost::none) {
        return boost::none;
    }
    return cpputils::unique_ref<Block>(std::move(*result));
}

template<class Compressor>
cpputils::unique_ref<Block> CompressingBlockStore<Compressor>::overwrite(const blockstore::BlockId &blockId, cpputils::Data data) {
    return CompressedBlock<Compressor>::Overwrite(_baseBlockStore.get(), blockId, std::move(data));
}

template<class Compressor>
boost::optional<cpputils::unique_ref<Block>> CompressingBlockStore<Compressor>::load(const BlockId &blockId) {
    auto loaded = _baseBlockStore->load(blockId);
    if (loaded == boost::none) {
        return boost::none;
    }
    return boost::optional<cpputils::unique_ref<Block>>(CompressedBlock<Compressor>::Decompress(std::move(*loaded)));
}

template<class Compressor>
void CompressingBlockStore<Compressor>::remove(const BlockId &blockId) {
    return _baseBlockStore->remove(blockId);
}

template<class Compressor>
uint64_t CompressingBlockStore<Compressor>::numBlocks() const {
    return _baseBlockStore->numBlocks();
}

template<class Compressor>
uint64_t CompressingBlockStore<Compressor>::estimateNumFreeBytes() const {
    return _baseBlockStore->estimateNumFreeBytes();
}

template<class Compressor>
void CompressingBlockStore<Compressor>::forEachBlock(std::function<void (const BlockId &)> callback) const {
    return _baseBlockStore->forEachBlock(callback);
}

template<class Compressor>
uint64_t CompressingBlockStore<Compressor>::blockSizeFromPhysicalBlockSize(uint64_t blockSize) const {
    //We probably have more since we're compressing, but we don't know exactly how much.
    //The best we can do is ignore the compression step here.
    return _baseBlockStore->blockSizeFromPhysicalBlockSize(blockSize);
}

}
}

#endif
