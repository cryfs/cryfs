#pragma once
#ifndef MESSMER_CRYFS_FILESYSTEM_RUSTFSBLOBSTORE_RUSTFILEBLOB_H_
#define MESSMER_CRYFS_FILESYSTEM_RUSTFSBLOBSTORE_RUSTFILEBLOB_H_

#include <cpp-utils/macros.h>
#include <cpp-utils/pointer/unique_ref.h>
#include <fspp/fs_interface/Types.h>
#include <blockstore/utils/BlockId.h>
#include <unordered_map>
#include "cxxbridge/cryfs-cppbridge/src/fsblobstore.rs.h"

namespace cryfs
{
    namespace fsblobstore
    {
        namespace rust
        {
            class RustFileBlob final
            {
            public:
                RustFileBlob(::rust::Box<bridge::RustFileBlobBridge> fileBlob);

                fspp::num_bytes_t read(void *target, fspp::num_bytes_t offset, fspp::num_bytes_t count);

                void write(const void *source, fspp::num_bytes_t offset, fspp::num_bytes_t count);

                void flush();

                void resize(fspp::num_bytes_t size);

                fspp::num_bytes_t size();

                blockstore::BlockId parent() const;

                blockstore::BlockId blockId() const;

            private:
                ::rust::Box<bridge::RustFileBlobBridge> _fileBlob;

                DISALLOW_COPY_AND_ASSIGN(RustFileBlob);
            };

        } // namespace rust
    }     // namespace blobstore
} // namespace cryfs

#endif
