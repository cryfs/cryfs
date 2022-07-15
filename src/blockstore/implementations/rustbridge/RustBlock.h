#pragma once
#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_RUSTBRIDGE_RUSTBLOCK_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_RUSTBRIDGE_RUSTBLOCK_H_

#include "../../interface/Block.h"
#include <cpp-utils/macros.h>
#include <unordered_map>
#include "cxxbridge/cryfs-cppbridge/src/blockstore.rs.h"

namespace blockstore
{
  namespace rust
  {

    class RustBlock final : public Block
    {
    public:
      RustBlock(::rust::Box<bridge::RustBlockBridge> block);
      virtual ~RustBlock();

      const void *data() const override;
      void write(const void *source, uint64_t offset, uint64_t size) override;

      void flush() override;

      size_t size() const override;

      void resize(size_t newSize) override;

    private:
      ::rust::Box<bridge::RustBlockBridge> _block;

      DISALLOW_COPY_AND_ASSIGN(RustBlock);
    };

  } // namespace rust
} // namespace blockstore

#endif
