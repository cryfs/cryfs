#pragma once
#ifndef MESSMER_BLOBSTORE_IMPLEMENTATIONS_RUSTBRIDGE_RUSTBLOBSTORE_H_
#define MESSMER_BLOBSTORE_IMPLEMENTATIONS_RUSTBRIDGE_RUSTBLOBSTORE_H_

#include "../../interface/BlobStore.h"
#include <cpp-utils/macros.h>
#include <unordered_map>
#include "cxxbridge/cryfs-cppbridge/src/blobstore.rs.h"

namespace blobstore
{
    namespace rust
    {

        class RustBlobStore final : public BlobStore
        {
        public:
            RustBlobStore(::rust::Box<bridge::RustBlobStoreBridge> blobStore);
            ~RustBlobStore();

            cpputils::unique_ref<Blob> create() override;
            boost::optional<cpputils::unique_ref<Blob>> load(const blockstore::BlockId &blockId) override;
            void remove(cpputils::unique_ref<Blob> blob) override;
            void remove(const blockstore::BlockId &blockId) override;

            uint64_t numBlocks() const override;
            uint64_t estimateSpaceForNumBlocksLeft() const override;
            // virtual means "space we can use" as opposed to "space it takes on the disk" (i.e. virtual is without headers, checksums, ...)
            uint64_t virtualBlocksizeBytes() const override;

        private:
            ::rust::Box<bridge::RustBlobStoreBridge> _blobStore;

            DISALLOW_COPY_AND_ASSIGN(RustBlobStore);
        };

    } // namespace rust
} // namespace blobstore

#endif
