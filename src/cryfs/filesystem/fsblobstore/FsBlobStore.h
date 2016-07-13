#pragma once
#ifndef MESSMER_CRYFS_FILESYSTEM_FSBLOBSTORE_FSBLOBSTORE_H
#define MESSMER_CRYFS_FILESYSTEM_FSBLOBSTORE_FSBLOBSTORE_H

#include <cpp-utils/lock/LockPool.h>
#include <cpp-utils/pointer/unique_ref.h>
#include <blobstore/interface/BlobStore.h>
#include "FileBlob.h"
#include "DirBlob.h"
#include "SymlinkBlob.h"

namespace cryfs {
    namespace fsblobstore {
        //TODO Test classes in fsblobstore

        class FsBlobStore final {
        public:
            FsBlobStore(cpputils::unique_ref<blobstore::BlobStore> baseBlobStore);

            cpputils::unique_ref<FileBlob> createFileBlob(const blockstore::Key &parent);
            cpputils::unique_ref<DirBlob> createDirBlob(const blockstore::Key &parent);
            cpputils::unique_ref<SymlinkBlob> createSymlinkBlob(const boost::filesystem::path &target, const blockstore::Key &parent);
            boost::optional<cpputils::unique_ref<FsBlob>> load(const blockstore::Key &key);
            void remove(cpputils::unique_ref<FsBlob> blob);
            void remove(const blockstore::Key &key);
            uint64_t numBlocks() const;
            uint64_t estimateSpaceForNumBlocksLeft() const;

            uint64_t virtualBlocksizeBytes() const;

#ifndef CRYFS_NO_COMPATIBILITY
            static cpputils::unique_ref<FsBlobStore> migrateIfNeeded(cpputils::unique_ref<blobstore::BlobStore> blobStore, const blockstore::Key &rootKey);
#endif

        private:

#ifndef CRYFS_NO_COMPATIBILITY
            void _migrate(cpputils::unique_ref<blobstore::Blob> node, const blockstore::Key &parentKey);
#endif

            std::function<off_t(const blockstore::Key &)> _getLstatSize();

            cpputils::unique_ref<blobstore::BlobStore> _baseBlobStore;

            DISALLOW_COPY_AND_ASSIGN(FsBlobStore);
        };

        inline FsBlobStore::FsBlobStore(cpputils::unique_ref<blobstore::BlobStore> baseBlobStore)
                : _baseBlobStore(std::move(baseBlobStore)) {
        }

        inline cpputils::unique_ref<FileBlob> FsBlobStore::createFileBlob(const blockstore::Key &parent) {
            auto blob = _baseBlobStore->create();
            return FileBlob::InitializeEmptyFile(std::move(blob), parent);
        }

        inline cpputils::unique_ref<DirBlob> FsBlobStore::createDirBlob(const blockstore::Key &parent) {
            auto blob = _baseBlobStore->create();
            return DirBlob::InitializeEmptyDir(this, std::move(blob), parent, _getLstatSize());
        }

        inline cpputils::unique_ref<SymlinkBlob> FsBlobStore::createSymlinkBlob(const boost::filesystem::path &target, const blockstore::Key &parent) {
            auto blob = _baseBlobStore->create();
            return SymlinkBlob::InitializeSymlink(std::move(blob), target, parent);
        }

        inline uint64_t FsBlobStore::numBlocks() const {
            return _baseBlobStore->numBlocks();
        }

        inline uint64_t FsBlobStore::estimateSpaceForNumBlocksLeft() const {
            return _baseBlobStore->estimateSpaceForNumBlocksLeft();
        }

        inline void FsBlobStore::remove(cpputils::unique_ref<FsBlob> blob) {
            _baseBlobStore->remove(blob->releaseBaseBlob());
        }

        inline void FsBlobStore::remove(const blockstore::Key &key) {
            _baseBlobStore->remove(key);
        }

        inline std::function<off_t (const blockstore::Key &)> FsBlobStore::_getLstatSize() {
            return [this] (const blockstore::Key &key) {
                auto blob = load(key);
                ASSERT(blob != boost::none, "Blob not found");
                return (*blob)->lstat_size();
            };
        }

        inline uint64_t FsBlobStore::virtualBlocksizeBytes() const {
            return _baseBlobStore->virtualBlocksizeBytes();
        }
    }
}

#endif
