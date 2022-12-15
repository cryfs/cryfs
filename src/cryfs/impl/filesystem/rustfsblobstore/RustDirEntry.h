#pragma once
#ifndef MESSMER_CRYFS_FILESYSTEM_RUSTFSBLOBSTORE_RUSTDIRENRTY_H_
#define MESSMER_CRYFS_FILESYSTEM_RUSTFSBLOBSTORE_RUSTDIRENTRY_H_

#include <cpp-utils/macros.h>
#include <cpp-utils/pointer/unique_ref.h>
#include <fspp/fs_interface/Types.h>
#include <fspp/fs_interface/Dir.h>
#include <blockstore/utils/BlockId.h>
#include <unordered_map>
#include "cxxbridge/cryfs-cppbridge/src/fsblobstore.rs.h"

namespace cryfs
{
    namespace fsblobstore
    {
        namespace rust
        {
            class RustDirEntry final
            {
            public:
                RustDirEntry(::rust::Box<bridge::RustDirEntryBridge> dirEntry);

                fspp::Dir::EntryType type() const;

                std::string name() const;

                blockstore::BlockId blockId() const;

                fspp::mode_t mode() const;

                fspp::uid_t uid() const;

                fspp::gid_t gid() const;

                timespec lastAccessTime() const;

                timespec lastModificationTime() const;

                timespec lastMetadataChangeTime() const;

            private:
                ::rust::Box<bridge::RustDirEntryBridge> _dirEntry;

                DISALLOW_COPY_AND_ASSIGN(RustDirEntry);
            };

        } // namespace rust
    }     // namespace blobstore
} // namespace cryfs

#endif
