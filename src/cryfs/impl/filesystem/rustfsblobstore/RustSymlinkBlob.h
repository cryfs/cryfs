#pragma once
#ifndef MESSMER_CRYFS_FILESYSTEM_RUSTFSBLOBSTORE_RUSTSYMLINKBLOB_H_
#define MESSMER_CRYFS_FILESYSTEM_RUSTFSBLOBSTORE_RUSTSYMLINKBLOB_H_

#include <cpp-utils/macros.h>
#include <cpp-utils/pointer/unique_ref.h>
#include <unordered_map>
#include <boost/filesystem/path.hpp>
#include <blockstore/utils/BlockId.h>
#include "cxxbridge/cryfs-cppbridge/src/fsblobstore.rs.h"

namespace cryfs
{
    namespace fsblobstore
    {
        namespace rust
        {
            class RustSymlinkBlob final
            {
            public:
                RustSymlinkBlob(::rust::Box<bridge::RustSymlinkBlobBridge> symlinkBlob);

                const boost::filesystem::path target();

                blockstore::BlockId parent() const;

                blockstore::BlockId blockId() const;

            private:
                ::rust::Box<bridge::RustSymlinkBlobBridge> _symlinkBlob;

                DISALLOW_COPY_AND_ASSIGN(RustSymlinkBlob);
            };

        } // namespace rust
    }     // namespace blobstore
} // namespace cryfs

#endif
