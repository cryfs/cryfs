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
    ~CompressingBlockStore();

    Key createKey() override;
    boost::optional<cpputils::unique_ref<Block>> tryCreate(const Key &key, cpputils::Data data) override;
    boost::optional<cpputils::unique_ref<Block>> load(const Key &key) override;
    cpputils::unique_ref<Block> overwrite(const blockstore::Key &key, cpputils::Data data) override;
    void remove(const Key &key) override;
    void removeIfExists(const Key &key) override;
    uint64_t numBlocks() const override;
    uint64_t estimateNumFreeBytes() const override;
    uint64_t blockSizeFromPhysicalBlockSize(uint64_t blockSize) const override;
    void forEachBlock(std::function<void (const Key &)> callback) const override;
    bool exists(const Key &key) const override;

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
Key CompressingBlockStore<Compressor>::createKey() {
    return _baseBlockStore->createKey();
}

template<class Compressor>
boost::optional<cpputils::unique_ref<Block>> CompressingBlockStore<Compressor>::tryCreate(const Key &key, cpputils::Data data) {
    auto result = CompressedBlock<Compressor>::TryCreateNew(_baseBlockStore.get(), key, std::move(data));
    if (result == boost::none) {
        return boost::none;
    }
    return cpputils::unique_ref<Block>(std::move(*result));
}

template<class Compressor>
cpputils::unique_ref<Block> CompressingBlockStore<Compressor>::overwrite(const blockstore::Key &key, cpputils::Data data) {
    return CompressedBlock<Compressor>::Overwrite(_baseBlockStore.get(), key, std::move(data));
}

template<class Compressor>
boost::optional<cpputils::unique_ref<Block>> CompressingBlockStore<Compressor>::load(const Key &key) {
    auto loaded = _baseBlockStore->load(key);
    if (loaded == boost::none) {
        return boost::none;
    }
    return boost::optional<cpputils::unique_ref<Block>>(CompressedBlock<Compressor>::Decompress(std::move(*loaded)));
}

template<class Compressor>
void CompressingBlockStore<Compressor>::remove(const Key &key) {
    return _baseBlockStore->remove(key);
}

template<class Compressor>
void CompressingBlockStore<Compressor>::removeIfExists(const Key &key) {
    return _baseBlockStore->removeIfExists(key);
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
void CompressingBlockStore<Compressor>::forEachBlock(std::function<void (const Key &)> callback) const {
    return _baseBlockStore->forEachBlock(callback);
}

template<class Compressor>
uint64_t CompressingBlockStore<Compressor>::blockSizeFromPhysicalBlockSize(uint64_t blockSize) const {
    //We probably have more since we're compressing, but we don't know exactly how much.
    //The best we can do is ignore the compression step here.
    return _baseBlockStore->blockSizeFromPhysicalBlockSize(blockSize);
}

template<class Compressor>
bool CompressingBlockStore<Compressor>::exists(const Key &key) const {
    return _baseBlockStore->exists(key);
}

}
}

#endif
