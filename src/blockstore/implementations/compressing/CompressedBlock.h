#pragma once
#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_COMPRESSING_COMPRESSEDBLOCK_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_COMPRESSING_COMPRESSEDBLOCK_H_

#include "../../interface/Block.h"
#include "../../interface/BlockStore.h"
#include <cpp-utils/data/DataUtils.h>
#include <cpp-utils/pointer/unique_ref.h>
#include <mutex>

namespace blockstore {
class BlockStore;
namespace compressing {
template<class Compressor> class CompressingBlockStore;

template<class Compressor>
class CompressedBlock final: public Block {
public:
  static boost::optional<cpputils::unique_ref<CompressedBlock>> TryCreateNew(BlockStore *baseBlockStore, const BlockId &blockId, cpputils::Data decompressedData);
  static cpputils::unique_ref<CompressedBlock> Overwrite(BlockStore *baseBlockStore, const BlockId &blockId, cpputils::Data decompressedData);
  static cpputils::unique_ref<CompressedBlock> Decompress(cpputils::unique_ref<Block> baseBlock);

  CompressedBlock(cpputils::unique_ref<Block> baseBlock, cpputils::Data decompressedData);
  ~CompressedBlock();

  const void *data() const override;
  void write(const void *source, uint64_t offset, uint64_t size) override;

  void flush() override;

  size_t size() const override;
  void resize(size_t newSize) override;

  cpputils::unique_ref<Block> releaseBaseBlock();

private:
  void _compressToBaseBlock();

  cpputils::unique_ref<Block> _baseBlock;
  cpputils::Data _decompressedData;
  std::mutex _mutex;
  bool _dataChanged;

  DISALLOW_COPY_AND_ASSIGN(CompressedBlock);
};

template<class Compressor>
boost::optional<cpputils::unique_ref<CompressedBlock<Compressor>>> CompressedBlock<Compressor>::TryCreateNew(BlockStore *baseBlockStore, const BlockId &blockId, cpputils::Data decompressedData) {
  cpputils::Data compressed = Compressor::Compress(decompressedData);
  auto baseBlock = baseBlockStore->tryCreate(blockId, std::move(compressed));
  if (baseBlock == boost::none) {
    //TODO Test this code branch
    return boost::none;
  }

  return cpputils::make_unique_ref<CompressedBlock<Compressor>>(std::move(*baseBlock), std::move(decompressedData));
}

template<class Compressor>
cpputils::unique_ref<CompressedBlock<Compressor>> CompressedBlock<Compressor>::Overwrite(BlockStore *baseBlockStore, const BlockId &blockId, cpputils::Data decompressedData) {
  cpputils::Data compressed = Compressor::Compress(decompressedData);
  auto baseBlock = baseBlockStore->overwrite(blockId, std::move(compressed));

  return cpputils::make_unique_ref<CompressedBlock<Compressor>>(std::move(baseBlock), std::move(decompressedData));
}

template<class Compressor>
cpputils::unique_ref<CompressedBlock<Compressor>> CompressedBlock<Compressor>::Decompress(cpputils::unique_ref<Block> baseBlock) {
  cpputils::Data decompressed = Compressor::Decompress(baseBlock->data(), baseBlock->size());
  return cpputils::make_unique_ref<CompressedBlock<Compressor>>(std::move(baseBlock), std::move(decompressed));
}

template<class Compressor>
CompressedBlock<Compressor>::CompressedBlock(cpputils::unique_ref<Block> baseBlock, cpputils::Data decompressedData)
        : Block(baseBlock->blockId()),
          _baseBlock(std::move(baseBlock)),
          _decompressedData(std::move(decompressedData)),
          _dataChanged(false) {
}

template<class Compressor>
CompressedBlock<Compressor>::~CompressedBlock() {
  std::unique_lock<std::mutex> lock(_mutex);
  _compressToBaseBlock();
}

template<class Compressor>
const void *CompressedBlock<Compressor>::data() const {
  return _decompressedData.data();
}

template<class Compressor>
void CompressedBlock<Compressor>::write(const void *source, uint64_t offset, uint64_t size) {
  std::memcpy(_decompressedData.dataOffset(offset), source, size);
  _dataChanged = true;
}

template<class Compressor>
void CompressedBlock<Compressor>::flush() {
  std::unique_lock<std::mutex> lock(_mutex);
  _compressToBaseBlock();
  return _baseBlock->flush();
}

template<class Compressor>
size_t CompressedBlock<Compressor>::size() const {
  return _decompressedData.size();
}

template<class Compressor>
void CompressedBlock<Compressor>::resize(size_t newSize) {
  _decompressedData = cpputils::DataUtils::resize(_decompressedData, newSize);
  _dataChanged = true;
}

template<class Compressor>
cpputils::unique_ref<Block> CompressedBlock<Compressor>::releaseBaseBlock() {
  std::unique_lock<std::mutex> lock(_mutex);
  _compressToBaseBlock();
  return std::move(_baseBlock);
}

template<class Compressor>
void CompressedBlock<Compressor>::_compressToBaseBlock() {
  if (_dataChanged) {
    cpputils::Data compressed = Compressor::Compress(_decompressedData);
    _baseBlock->resize(compressed.size());
    _baseBlock->write(compressed.data(), 0, compressed.size());
    _dataChanged = false;
  }
}

}
}

#endif
