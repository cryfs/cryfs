#pragma once
#ifndef MESSMER_CRYFS_FILESYSTEM_RUSTFSBLOBSTORE_RUSTFSBLOBSTORE_H_
#define MESSMER_CRYFS_FILESYSTEM_RUSTFSBLOBSTORE_RUSTFSBLOBSTORE_H_

#include <unordered_map>

#include <cpp-utils/macros.h>
#include <cpp-utils/pointer/unique_ref.h>
#include <blockstore/utils/BlockId.h>
#include "cxxbridge/cryfs-cppbridge/src/fsblobstore.rs.h"

#include "RustFsBlob.h"
#include "RustDirBlob.h"
#include "RustFileBlob.h"
#include "RustSymlinkBlob.h"

namespace cryfs
{
    namespace fsblobstore
    {
        namespace rust
        {
            class RustFsBlobStore final
            {
            public:
                RustFsBlobStore(::rust::Box<bridge::RustFsBlobStoreBridge> fsBlobStore);
                ~RustFsBlobStore();
                cpputils::unique_ref<RustDirBlob> createDirBlob(const blockstore::BlockId &parent);
                cpputils::unique_ref<RustFileBlob> createFileBlob(const blockstore::BlockId &parent);
                cpputils::unique_ref<RustSymlinkBlob> createSymlinkBlob(const boost::filesystem::path &target, const blockstore::BlockId &parent);
                boost::optional<cpputils::unique_ref<RustFsBlob>> load(const blockstore::BlockId &blockId);
                uint64_t numBlocks() const;
                uint64_t estimateSpaceForNumBlocksLeft() const;
                uint64_t virtualBlocksizeBytes() const;
                uint8_t loadBlockDepth(const blockstore::BlockId &blockId) const;

            private:
                ::rust::Box<bridge::RustFsBlobStoreBridge> _fsBlobStore;

                DISALLOW_COPY_AND_ASSIGN(RustFsBlobStore);
            };

        } // namespace rust
    }     // namespace blobstore
} // namespace cryfs

#endif
