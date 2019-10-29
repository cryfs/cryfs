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
#include <shared_mutex>


namespace cryfs {
    namespace fsblobstore {
        class FsBlobStore;

        class DirBlob final : public FsBlob {
        public:

            static cpputils::unique_ref<DirBlob> InitializeEmptyDir(cpputils::unique_ref<blobstore::Blob> blob,
                                                                    const FsBlobView::Metadata &meta, const TimestampUpdateBehavior&);

            explicit DirBlob(cpputils::unique_ref<blobstore::Blob> blob, const TimestampUpdateBehavior& behav);

            ~DirBlob() override;


            void AppendChildrenTo(std::vector<fspp::Dir::Entry> *result) const;

            //TODO Test NumChildren()
            size_t NumChildren() const;

            boost::optional<const DirEntry&> GetChild(const std::string &name) const;

            boost::optional<const DirEntry&> GetChild(const blockstore::BlockId &blobId) const;

            void AddChildDir(const std::string &name, const blockstore::BlockId &blobId);

            void AddChildFile(const std::string &name, const blockstore::BlockId &blobId);

            void AddChildSymlink(const std::string &name, const blockstore::BlockId &blobId);

            void AddChildHardlink(const std::string& name, const blockstore::BlockId &blobId, fspp::Dir::EntryType type);

            void AddOrOverwriteChild(const std::string &name, const blockstore::BlockId &blobId, fspp::Dir::EntryType type,
                          const std::function<void (const DirEntry &entry)>& onOverwritten);

            void RenameChild(const blockstore::BlockId &blockId, const std::string &newName, const std::function<void (const DirEntry &entry)>& onOverwritten);

            void RemoveChild(const std::string &name);

            void RemoveChild(const blockstore::BlockId &blockId);

            void flush();

          void utimens(timespec atime, timespec mtime);


        private:

            void _addChild(const std::string &name, const blockstore::BlockId &blobId, fspp::Dir::EntryType type);
            void _readEntriesFromBlob();
            void _writeEntriesToBlob();

            cpputils::unique_ref<blobstore::Blob> releaseBaseBlob() override;

            DirEntryList _entries;
            // TODO: switch to c++17 and use shared_mutex
            mutable std::shared_timed_mutex _entriesAndChangedMutex;
            bool _changed;

            DISALLOW_COPY_AND_ASSIGN(DirBlob);
        };

    }
}

#endif
