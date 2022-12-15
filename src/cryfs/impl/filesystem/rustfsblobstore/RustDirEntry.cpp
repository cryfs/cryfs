#include "RustDirEntry.h"
#include <cryfs/impl/filesystem/rustfsblobstore/helpers.h>

using blockstore::BlockId;
using cryfs::fsblobstore::rust::bridge::RustTimespec;
using cryfs::fsblobstore::rust::helpers::cast_blobid;
using cryfs::fsblobstore::rust::helpers::cast_timespec;
using cryfs::fsblobstore::rust::helpers::cast_entry_type;

namespace cryfs
{
    namespace fsblobstore
    {
        namespace rust
        {
            RustDirEntry::RustDirEntry(::rust::Box<bridge::RustDirEntryBridge> dirEntry)
                : _dirEntry(std::move(dirEntry))
            {
            }

            fspp::Dir::EntryType RustDirEntry::type() const
            {
                return cast_entry_type(_dirEntry->entry_type());
            }

            std::string RustDirEntry::name() const
            {
                return static_cast<std::string>(_dirEntry->name());
            }

            blockstore::BlockId RustDirEntry::blockId() const
            {
                return cast_blobid(*_dirEntry->blob_id());
            }

            fspp::mode_t RustDirEntry::mode() const
            {
                return fspp::mode_t(_dirEntry->mode());
            }

            fspp::uid_t RustDirEntry::uid() const
            {
                return fspp::uid_t(_dirEntry->uid());
            }

            fspp::gid_t RustDirEntry::gid() const
            {
                return fspp::gid_t(_dirEntry->gid());
            }

            timespec RustDirEntry::lastAccessTime() const
            {
                return helpers::cast_timespec(_dirEntry->last_access_time());
            }

            timespec RustDirEntry::lastModificationTime() const
            {
                return helpers::cast_timespec(_dirEntry->last_modification_time());
            }

            timespec RustDirEntry::lastMetadataChangeTime() const
            {
                return helpers::cast_timespec(_dirEntry->last_metadata_change_time());
            }
        }
    }
}