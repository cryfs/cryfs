#pragma once
#ifndef MESSMER_CRYFS_FILESYSTEM_RUSTFSBLOBSTORE_HELPERS_H_
#define MESSMER_CRYFS_FILESYSTEM_RUSTFSBLOBSTORE_HELPERS_H_

#include "cxxbridge/cryfs-cppbridge/src/fsblobstore.rs.h"
#include <fspp/fs_interface/Dir.h>

namespace cryfs
{
    namespace fsblobstore
    {
        namespace rust
        {
            namespace helpers
            {
                inline ::rust::Box<bridge::FsBlobId> cast_blobid(const blockstore::BlockId &blobId)
                {
                    return bridge::new_blobid(blobId.data().as_array());
                }

                inline blockstore::BlockId cast_blobid(const bridge::FsBlobId &blobId)
                {
                    return blockstore::BlockId::FromBinary(blobId.data().data());
                }

                inline struct timespec cast_timespec(bridge::RustTimespec value)
                {
                    return {static_cast<time_t>(value.tv_sec), value.tv_nsec};
                }

                inline bridge::RustTimespec cast_timespec(struct timespec value)
                {
                    return {static_cast<uint64_t>(value.tv_sec), static_cast<uint32_t>(value.tv_nsec)};
                }

                inline fspp::Dir::EntryType cast_entry_type(bridge::RustEntryType value)
                {
                    switch (value)
                    {
                    case bridge::RustEntryType::File:
                        return fspp::Dir::EntryType::FILE;
                    case bridge::RustEntryType::Dir:
                        return fspp::Dir::EntryType::DIR;
                    case bridge::RustEntryType::Symlink:
                        return fspp::Dir::EntryType::SYMLINK;
                    default:
                        throw std::runtime_error("Unknown entry type");
                    }
                }

                inline bridge::RustEntryType cast_entry_type(fspp::Dir::EntryType value)
                {
                    switch (value)
                    {
                    case fspp::Dir::EntryType::FILE:
                        return bridge::RustEntryType::File;
                    case fspp::Dir::EntryType::DIR:
                        return bridge::RustEntryType::Dir;
                    case fspp::Dir::EntryType::SYMLINK:
                        return bridge::RustEntryType::Symlink;
                    default:
                        throw std::runtime_error("Unknown entry type");
                    }
                }

                inline fspp::Dir::Entry cast_entry(const bridge::RustDirEntryBridge &entry)
                {
                    return fspp::Dir::Entry(
                        cast_entry_type(entry.entry_type()),
                        static_cast<std::string>(entry.name())
                    );
                }
            }
        }
    }
}

#endif
