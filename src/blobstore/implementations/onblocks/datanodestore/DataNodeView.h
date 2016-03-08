#pragma once
#ifndef MESSMER_BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_DATANODESTORE_DATANODEVIEW_H_
#define MESSMER_BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_DATANODESTORE_DATANODEVIEW_H_

#include <blockstore/interface/Block.h>
#include "../BlobStoreOnBlocks.h"
#include "DataInnerNode_ChildEntry.h"

#include <cpp-utils/pointer/unique_ref.h>

#include <memory>
#include <stdexcept>
#include <type_traits>

namespace blobstore {
namespace onblocks {
namespace datanodestore {

//TODO Move DataNodeLayout into own file
class DataNodeLayout final {
public:
  constexpr DataNodeLayout(uint64_t blocksizeBytes)
    :_blocksizeBytes(
        (HEADERSIZE_BYTES + 2*sizeof(DataInnerNode_ChildEntry) <= blocksizeBytes)
        ? blocksizeBytes
        : throw std::logic_error("Blocksize too small, not enough space to store two children in an inner node")) {
  }

  //Total size of the header
  static constexpr uint32_t HEADERSIZE_BYTES = 8;
  //Where in the header is the format version field (used to allow compatibility with future versions of CryFS)
  static constexpr uint32_t FORMAT_VERSION_OFFSET_BYTES = 0; //format version uses 2 bytes
  //Where in the header is the depth field
  static constexpr uint32_t DEPTH_OFFSET_BYTES = 3; // depth uses 1 byte
  //Where in the header is the size field (for inner nodes: number of children, for leafs: content data size)
  static constexpr uint32_t SIZE_OFFSET_BYTES = 4; // size uses 4 bytes


  //Size of a block (header + data region)
  constexpr uint64_t blocksizeBytes() const {
    return _blocksizeBytes;
  }

  //Number of bytes in the data region of a node
  constexpr uint64_t datasizeBytes() const {
    return _blocksizeBytes - HEADERSIZE_BYTES;
  }

  //Maximum number of children an inner node can store
  constexpr uint64_t maxChildrenPerInnerNode() const {
    return datasizeBytes() / sizeof(DataInnerNode_ChildEntry);
  }

  //Maximum number of bytes a leaf can store
  constexpr uint64_t maxBytesPerLeaf() const {
    return datasizeBytes();
  }
private:
  uint32_t _blocksizeBytes;
};

class DataNodeView final {
public:
  DataNodeView(cpputils::unique_ref<blockstore::Block> block): _block(std::move(block)) {
  }
  ~DataNodeView() {}

  DataNodeView(DataNodeView &&rhs) = default;

  uint16_t FormatVersion() const {
    return *((uint8_t*)_block->data()+DataNodeLayout::FORMAT_VERSION_OFFSET_BYTES);
  }

  void setFormatVersion(uint16_t value) {
    _block->write(&value, DataNodeLayout::FORMAT_VERSION_OFFSET_BYTES, sizeof(value));
  }

  uint8_t Depth() const {
    return *((uint8_t*)_block->data()+DataNodeLayout::DEPTH_OFFSET_BYTES);
  }

  void setDepth(uint8_t value) {
    _block->write(&value, DataNodeLayout::DEPTH_OFFSET_BYTES, sizeof(value));
  }

  uint32_t Size() const {
    return *(uint32_t*)((uint8_t*)_block->data()+DataNodeLayout::SIZE_OFFSET_BYTES);
  }

  void setSize(uint32_t value) {
    _block->write(&value, DataNodeLayout::SIZE_OFFSET_BYTES, sizeof(value));
  }

  const void *data() const {
    return (uint8_t*)_block->data() + DataNodeLayout::HEADERSIZE_BYTES;
  }

  void write(const void *source, uint64_t offset, uint64_t size) {
    _block->write(source, offset + DataNodeLayout::HEADERSIZE_BYTES, size);
  }

  template<typename Entry>
  const Entry *DataBegin() const {
    return GetOffset<DataNodeLayout::HEADERSIZE_BYTES, Entry>();
  }

  template<typename Entry>
  const Entry *DataEnd() const {
    const unsigned int NUM_ENTRIES = layout().datasizeBytes() / sizeof(Entry);
    return DataBegin<Entry>() + NUM_ENTRIES;
  }

  DataNodeLayout layout() const {
    return DataNodeLayout(_block->size());
  }

  cpputils::unique_ref<blockstore::Block> releaseBlock() {
    return std::move(_block);
  }

  const blockstore::Block &block() const {
    return *_block;
  }

  const blockstore::Key &key() const {
    return _block->key();
  }

  void flush() const {
    _block->flush();
  }

private:
  template<int offset, class Type>
  const Type *GetOffset() const {
    return (Type*)(((const int8_t*)_block->data())+offset);
  }

  cpputils::unique_ref<blockstore::Block> _block;

  DISALLOW_COPY_AND_ASSIGN(DataNodeView);

};

}
}
}

#endif
