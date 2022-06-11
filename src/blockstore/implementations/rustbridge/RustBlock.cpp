#include "RustBlock.h"
#include "helpers.h"

namespace blockstore
{
    namespace rust
    {
      RustBlock::RustBlock(::rust::Box<bridge::RustBlockBridge> block)
      : Block(helpers::cast_blockid(*block->block_id())), _block(std::move(block)) {}

      RustBlock::~RustBlock() {
          _block->async_drop();
      }

      const void *RustBlock::data() const {
          return _block->data().data();
      }

      void RustBlock::write(const void *source, uint64_t offset, uint64_t size) {
          auto source_slice = ::rust::Slice<const uint8_t>{static_cast<const uint8_t *>(source), size};
          _block->write(source_slice, offset);
      }

      void RustBlock::flush() {
          _block->flush();
      }

      size_t RustBlock::size() const {
          return _block->size();
      }

      void RustBlock::resize(size_t newSize) {
          _block->resize(newSize);
      }
    }
}
