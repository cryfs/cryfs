#pragma once
#ifndef BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_DATANODESTORE_DATANODEVIEW_H_
#define BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_DATANODESTORE_DATANODEVIEW_H_

#include "blockstore/interface/Block.h"
#include "blobstore/implementations/onblocks/BlobStoreOnBlocks.h"

#include "fspp/utils/macros.h"

#include <memory>
#include <cassert>

namespace blobstore {
namespace onblocks {
namespace datanodestore {

class DataNodeView {
public:
  DataNodeView(std::unique_ptr<blockstore::Block> block): _block(std::move(block)) {
    assert(_block->size() == BLOCKSIZE_BYTES);
  }
  virtual ~DataNodeView() {}

  DataNodeView(DataNodeView &&rhs) = default;

  //Total size of the header
  static constexpr unsigned int HEADERSIZE_BYTES = 8;
  //Where in the header is the depth field
  static constexpr unsigned int DEPTH_OFFSET_BYTES = 0;
  //Where in the header is the size field (for inner nodes: number of children, for leafs: content data size)
  static constexpr unsigned int SIZE_OFFSET_BYTES = 4;

  //How big is one blob in total (header + data)
  static constexpr unsigned int BLOCKSIZE_BYTES = BlobStoreOnBlocks::BLOCKSIZE;
  //How much space is there for data
  static constexpr unsigned int DATASIZE_BYTES = BLOCKSIZE_BYTES - HEADERSIZE_BYTES;

  const uint8_t *Depth() const {
    return GetOffset<DEPTH_OFFSET_BYTES, uint8_t>();
  }

  uint8_t *Depth() {
    return const_cast<uint8_t*>(const_cast<const DataNodeView*>(this)->Depth());
  }

  const uint32_t *Size() const {
    return GetOffset<SIZE_OFFSET_BYTES, uint32_t>();
  }

  uint32_t *Size() {
    return const_cast<uint32_t*>(const_cast<const DataNodeView*>(this)->Size());
  }

  template<typename Entry>
  const Entry *DataBegin() const {
    return GetOffset<HEADERSIZE_BYTES, Entry>();
  }

  template<typename Entry>
  Entry *DataBegin() {
    return const_cast<Entry*>(const_cast<const DataNodeView*>(this)->DataBegin<Entry>());
  }

  template<typename Entry>
  const Entry *DataEnd() const {
    constexpr unsigned int NUM_ENTRIES = DATASIZE_BYTES / sizeof(Entry);
    return DataBegin<Entry>() + NUM_ENTRIES;
  }

  template<typename Entry>
  Entry *DataEnd() {
    return const_cast<Entry*>(const_cast<const DataNodeView*>(this)->DataEnd<Entry>());
  }

  std::unique_ptr<blockstore::Block> releaseBlock() {
    return std::move(_block);
  }

private:
  template<int offset, class Type>
  const Type *GetOffset() const {
    return (Type*)(((const int8_t*)_block->data())+offset);
  }

  std::unique_ptr<blockstore::Block> _block;

  DISALLOW_COPY_AND_ASSIGN(DataNodeView);

};

}
}
}

#endif
