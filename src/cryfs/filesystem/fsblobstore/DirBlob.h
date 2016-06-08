#pragma once
#ifndef MESSMER_CRYFS_FILESYSTEM_FSBLOBSTORE_DIRBLOB_H_
#define MESSMER_CRYFS_FILESYSTEM_FSBLOBSTORE_DIRBLOB_H_

#include <blockstore/utils/Key.h>
#include <cpp-utils/macros.h>
#include <fspp/fs_interface/Dir.h>
#include "FsBlob.h"
#include "utils/DirEntryList.h"
#include <mutex>

namespace cryfs {
    namespace fsblobstore {
        class FsBlobStore;

        class DirBlob final : public FsBlob {
        public:
            constexpr static off_t DIR_LSTAT_SIZE = 4096;

            static cpputils::unique_ref<DirBlob> InitializeEmptyDir(FsBlobStore *fsBlobStore, cpputils::unique_ref<blobstore::Blob> blob,
                                                                    std::function<off_t (const blockstore::Key&)> getLstatSize);

            DirBlob(FsBlobStore *fsBlobStore, cpputils::unique_ref<blobstore::Blob> blob, std::function<off_t (const blockstore::Key&)> getLstatSize);

            ~DirBlob();

            off_t lstat_size() const override;

            void AppendChildrenTo(std::vector<fspp::Dir::Entry> *result) const;

            //TODO Test NumChildren()
            size_t NumChildren() const;

            boost::optional<const DirEntry&> GetChild(const std::string &name) const;

            boost::optional<const DirEntry&> GetChild(const blockstore::Key &key) const;

            void AddChildDir(const std::string &name, const blockstore::Key &blobKey, mode_t mode, uid_t uid,
                             gid_t gid, timespec lastAccessTime, timespec lastModificationTime);

            void AddChildFile(const std::string &name, const blockstore::Key &blobKey, mode_t mode, uid_t uid,
                              gid_t gid, timespec lastAccessTime, timespec lastModificationTime);

            void AddChildSymlink(const std::string &name, const blockstore::Key &blobKey, uid_t uid, gid_t gid, timespec lastAccessTime, timespec lastModificationTime);

            void AddOrOverwriteChild(const std::string &name, const blockstore::Key &blobKey, fspp::Dir::EntryType type,
                          mode_t mode, uid_t uid, gid_t gid, timespec lastAccessTime, timespec lastModificationTime,
                          std::function<void (const blockstore::Key &key)> onOverwritten);

            void RenameChild(const blockstore::Key &key, const std::string &newName, std::function<void (const blockstore::Key &key)> onOverwritten);

            void RemoveChild(const std::string &name);

            void RemoveChild(const blockstore::Key &key);

            void flush();

            void statChild(const blockstore::Key &key, struct ::stat *result) const;

            void statChildExceptSize(const blockstore::Key &key, struct ::stat *result) const;

            void updateAccessTimestampForChild(const blockstore::Key &key);

            void updateModificationTimestampForChild(const blockstore::Key &key);

            void chmodChild(const blockstore::Key &key, mode_t mode);

            void chownChild(const blockstore::Key &key, uid_t uid, gid_t gid);

            void utimensChild(const blockstore::Key &key, timespec lastAccessTime, timespec lastModificationTime);

            void setLstatSizeGetter(std::function<off_t(const blockstore::Key&)> getLstatSize);

        private:

            void _addChild(const std::string &name, const blockstore::Key &blobKey, fspp::Dir::EntryType type,
                          mode_t mode, uid_t uid, gid_t gid, timespec lastAccessTime, timespec lastModificationTime);
            void _readEntriesFromBlob();
            void _writeEntriesToBlob();

            cpputils::unique_ref<blobstore::Blob> releaseBaseBlob() override;

            FsBlobStore *_fsBlobStore;
            std::function<off_t (const blockstore::Key&)> _getLstatSize;
            DirEntryList _entries;
            mutable std::mutex _mutex;
            bool _changed;

            DISALLOW_COPY_AND_ASSIGN(DirBlob);
        };

    }
}

#endif
