#pragma once
#ifndef MESSMER_CRYFS_FILESYSTEM_RUSTFSBLOBSTORE_RUSTDIRBLOB_H_
#define MESSMER_CRYFS_FILESYSTEM_RUSTFSBLOBSTORE_RUSTDIRBLOB_H_

#include <cpp-utils/macros.h>
#include <cpp-utils/pointer/unique_ref.h>
#include <blockstore/utils/BlockId.h>
#include <unordered_map>
#include "cxxbridge/cryfs-cppbridge/src/fsblobstore.rs.h"
#include "RustDirEntry.h"
#include <fspp/fs_interface/Context.h>

namespace cryfs
{
    namespace fsblobstore
    {
        namespace rust
        {
            class RustDirBlob final
            {
            public:
                RustDirBlob(::rust::Box<bridge::RustDirBlobBridge> dirBlob);
                ~RustDirBlob();

                void flush();
                blockstore::BlockId blockId() const;
                blockstore::BlockId parent() const;
                size_t NumChildren() const;
                void AppendChildrenTo(std::vector<fspp::Dir::Entry> *result) const;

                void RenameChild(const blockstore::BlockId &blockId, const std::string &newName, std::function<void(const blockstore::BlockId &blockId)> onOverwritten);

                boost::optional<cpputils::unique_ref<RustDirEntry>> GetChild(const std::string &name) const;
                boost::optional<cpputils::unique_ref<RustDirEntry>> GetChild(const blockstore::BlockId &blockId) const;

                void maybeUpdateAccessTimestampOfChild(const blockstore::BlockId& blockId, fspp::TimestampUpdateBehavior atimeUpdateBehavior);
                void updateModificationTimestampOfChild(const blockstore::BlockId &blockId);
                void setModeOfChild(const blockstore::BlockId &blockId, fspp::mode_t mode);
                void setUidGidOfChild(const blockstore::BlockId &blockId, fspp::uid_t uid, fspp::gid_t gid);
                void setAccessTimesOfChild(const blockstore::BlockId &blockId, const timespec &lastAccessTime, const timespec &lastModificationTime);
                void AddChildDir(const std::string &name, const blockstore::BlockId &blobId, fspp::mode_t mode, fspp::uid_t uid, fspp::gid_t gid, timespec lastAccessTime, timespec lastModificationTime);
                void AddChildFile(const std::string &name, const blockstore::BlockId &blobId, fspp::mode_t mode, fspp::uid_t uid, fspp::gid_t gid, timespec lastAccessTime, timespec lastModificationTime);
                void AddChildSymlink(const std::string &name, const blockstore::BlockId &blobId, fspp::uid_t uid, fspp::gid_t gid, timespec lastAccessTime, timespec lastModificationTime);
                void AddOrOverwriteChild(const std::string &name, const blockstore::BlockId &blobId, fspp::Dir::EntryType entryType,
                                  fspp::mode_t mode, fspp::uid_t uid, fspp::gid_t gid, timespec lastAccessTime, timespec lastModificationTime,
                                  std::function<void (const blockstore::BlockId &blockId)> onOverwritten);
                void RemoveChild(const std::string &name);
                void RemoveChildIfExists(const blockstore::BlockId &blockId);

            private:
                ::rust::Box<bridge::RustDirBlobBridge> _dirBlob;

                DISALLOW_COPY_AND_ASSIGN(RustDirBlob);
            };

        } // namespace rust
    }     // namespace blobstore
} // namespace cryfs

#endif
