#pragma once
#ifndef MESSMER_CRYFS_FILESYSTEM_RUSTFSBLOBSTORE_RUSTFSBLOB_H_
#define MESSMER_CRYFS_FILESYSTEM_RUSTFSBLOBSTORE_RUSTFSBLOB_H_

#include <cpp-utils/macros.h>
#include <cpp-utils/pointer/unique_ref.h>
#include <unordered_map>
#include "cxxbridge/cryfs-cppbridge/src/fsblobstore.rs.h"

#include "RustFileBlob.h"
#include "RustDirBlob.h"
#include "RustSymlinkBlob.h"

namespace cryfs
{
    namespace fsblobstore
    {
        namespace rust
        {
            class RustFsBlob final
            {
            public:
                RustFsBlob(::rust::Box<bridge::RustFsBlobBridge> fsBlob);
                ~RustFsBlob();

                bool isFile() const;
                bool isDir() const;
                bool isSymlink() const;

                cpputils::unique_ref<RustFileBlob> asFile() &&;
                cpputils::unique_ref<RustDirBlob> asDir() &&;
                cpputils::unique_ref<RustSymlinkBlob> asSymlink() &&;

                blockstore::BlockId parent() const;
                void setParent(const blockstore::BlockId &parent);
                blockstore::BlockId blockId() const;

                fspp::num_bytes_t lstat_size();

                void remove() &&;

                std::vector<blockstore::BlockId> allBlocks() const;

            private:
                boost::optional<::rust::Box<bridge::RustFsBlobBridge>> _fsBlob;

                DISALLOW_COPY_AND_ASSIGN(RustFsBlob);
            };

        } // namespace rust
    }     // namespace blobstore
} // namespace cryfs

#endif
