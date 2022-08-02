#pragma once
#ifndef MESSMER_BLOBSTORE_IMPLEMENTATIONS_RUSTBRIDGE_RUSTBLOB_H_
#define MESSMER_BLOBSTORE_IMPLEMENTATIONS_RUSTBRIDGE_RUSTBLOB_H_

#include "../../interface/Blob.h"
#include <cpp-utils/macros.h>
#include <unordered_map>
#include "cxxbridge/cryfs-cppbridge/src/blobstore.rs.h"

namespace blobstore
{
    namespace rust
    {

        class RustBlob final : public Blob
        {
        public:
            RustBlob(::rust::Box<bridge::RustBlobBridge> blob);
            ~RustBlob();

            const blockstore::BlockId &blockId() const override;
            uint64_t size() const override;
            void resize(uint64_t numBytes) override;
            cpputils::Data readAll() const override;
            void read(void *target, uint64_t offset, uint64_t size) const override;
            uint64_t tryRead(void *target, uint64_t offset, uint64_t size) const override;
            void write(const void *source, uint64_t offset, uint64_t size) override;
            void flush() override;
            uint32_t numNodes() const override;
            void remove();

        private:
            ::rust::Box<bridge::RustBlobBridge> _blob;
            const blockstore::BlockId _blobId;

            DISALLOW_COPY_AND_ASSIGN(RustBlob);
        };

    } // namespace rust
} // namespace blobstore

#endif
