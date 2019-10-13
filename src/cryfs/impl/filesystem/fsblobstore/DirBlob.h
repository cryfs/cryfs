#pragma once
#ifndef MESSMER_CRYFS_FILESYSTEM_FSBLOBSTORE_DIRBLOB_H_
#define MESSMER_CRYFS_FILESYSTEM_FSBLOBSTORE_DIRBLOB_H_

#include <blockstore/utils/BlockId.h>
#include <cpp-utils/macros.h>
#include <fspp/fs_interface/Dir.h>
#include <fspp/fs_interface/Node.h>
#include "FsBlob.h"
#include "cryfs/impl/filesystem/fsblobstore/utils/DirEntryList.h"
#include <mutex>

namespace cryfs {
    namespace fsblobstore {
        class FsBlobStore;

        class DirBlob final : public FsBlob {
        public:
            constexpr static fspp::num_bytes_t DIR_LSTAT_SIZE = fspp::num_bytes_t(4096);

            static cpputils::unique_ref<DirBlob> InitializeEmptyDir(cpputils::unique_ref<blobstore::Blob> blob,
                                                                    const blockstore::BlockId &parent,
                                                                    std::function<fspp::num_bytes_t (const blockstore::BlockId&)> getLstatSize);

            DirBlob(cpputils::unique_ref<blobstore::Blob> blob, std::function<fspp::num_bytes_t (const blockstore::BlockId&)> getLstatSize);

            ~DirBlob();

            fspp::num_bytes_t lstat_size() const override;

            void AppendChildrenTo(std::vector<fspp::Dir::Entry> *result) const;

            //TODO Test NumChildren()
            size_t NumChildren() const;

            boost::optional<const DirEntry&> GetChild(const std::string &name) const;

            boost::optional<const DirEntry&> GetChild(const blockstore::BlockId &blobId) const;

            void AddChildDir(const std::string &name, const blockstore::BlockId &blobId, fspp::mode_t mode, fspp::uid_t uid,
                             fspp::gid_t gid, timespec lastAccessTime, timespec lastModificationTime);

            void AddChildFile(const std::string &name, const blockstore::BlockId &blobId, fspp::mode_t mode, fspp::uid_t uid,
                              fspp::gid_t gid, timespec lastAccessTime, timespec lastModificationTime);

            void AddChildSymlink(const std::string &name, const blockstore::BlockId &blobId, fspp::uid_t uid, fspp::gid_t gid, timespec lastAccessTime, timespec lastModificationTime);

            void AddOrOverwriteChild(const std::string &name, const blockstore::BlockId &blobId, fspp::Dir::EntryType type,
                          fspp::mode_t mode, fspp::uid_t uid, fspp::gid_t gid, timespec lastAccessTime, timespec lastModificationTime,
                          std::function<void (const blockstore::BlockId &blockId)> onOverwritten);

            void RenameChild(const blockstore::BlockId &blockId, const std::string &newName, std::function<void (const blockstore::BlockId &blockId)> onOverwritten);

            void RemoveChild(const std::string &name);

            void RemoveChild(const blockstore::BlockId &blockId);

            void flush();

            fspp::Node::stat_info statChild(const blockstore::BlockId &blockId) const;

            fspp::Node::stat_info statChildWithKnownSize(const blockstore::BlockId &blockId, fspp::num_bytes_t size) const;

            void updateAccessTimestampForChild(const blockstore::BlockId &blockId, fspp::TimestampUpdateBehavior timestampUpdateBehavior);

            void updateModificationTimestampForChild(const blockstore::BlockId &blockId);

            void chmodChild(const blockstore::BlockId &blockId, fspp::mode_t mode);

            void chownChild(const blockstore::BlockId &blockId, fspp::uid_t uid, fspp::gid_t gid);

            void utimensChild(const blockstore::BlockId &blockId, timespec lastAccessTime, timespec lastModificationTime);

            void setLstatSizeGetter(std::function<fspp::num_bytes_t(const blockstore::BlockId&)> getLstatSize);

        private:

            void _addChild(const std::string &name, const blockstore::BlockId &blobId, fspp::Dir::EntryType type,
                          fspp::mode_t mode, fspp::uid_t uid, fspp::gid_t gid, timespec lastAccessTime, timespec lastModificationTime);
            void _readEntriesFromBlob();
            void _writeEntriesToBlob();

            cpputils::unique_ref<blobstore::Blob> releaseBaseBlob() override;

            std::function<fspp::num_bytes_t (const blockstore::BlockId&)> _getLstatSize;
            mutable std::mutex _getLstatSizeMutex;
            DirEntryList _entries;
            mutable std::mutex _entriesAndChangedMutex;
            bool _changed;

            DISALLOW_COPY_AND_ASSIGN(DirBlob);
        };

    }
}

#endif
