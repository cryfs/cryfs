#pragma once
#ifndef MESSMER_CRYFS_FILESYSTEM_FSBLOBSTORE_FSBLOBSTORE_H
#define MESSMER_CRYFS_FILESYSTEM_FSBLOBSTORE_FSBLOBSTORE_H

#include <cpp-utils/lock/LockPool.h>
#include <cpp-utils/pointer/unique_ref.h>
#include <blobstore/interface/BlobStore.h>
#include "FileBlob.h"
#include "DirBlob.h"
#include "SymlinkBlob.h"
#ifndef CRYFS_NO_COMPATIBILITY
#include <cpp-utils/process/SignalCatcher.h>
#endif

namespace cryfs {
    namespace fsblobstore {
        //TODO Test classes in fsblobstore

        class FsBlobStore final {
        public:
            explicit FsBlobStore(cpputils::unique_ref<blobstore::BlobStore> baseBlobStore);

            cpputils::unique_ref<FileBlob> createFileBlob(const FsBlobView::Metadata &meta);
            cpputils::unique_ref<DirBlob> createDirBlob(const FsBlobView::Metadata &meta);
            cpputils::unique_ref<SymlinkBlob> createSymlinkBlob(const boost::filesystem::path &target, const FsBlobView::Metadata &meta);
            boost::optional<cpputils::unique_ref<FsBlob>> load(const blockstore::BlockId &blockId);
            void remove(cpputils::unique_ref<FsBlob> blob);
            void remove(const blockstore::BlockId &blockId);
            uint64_t numBlocks() const;
            uint64_t estimateSpaceForNumBlocksLeft() const;

            uint64_t virtualBlocksizeBytes() const;

#ifndef CRYFS_NO_COMPATIBILITY
            static cpputils::unique_ref<FsBlobStore> migrate(cpputils::unique_ref<blobstore::BlobStore> blobStore, const blockstore::BlockId &blockId);
#endif

        private:

#ifndef CRYFS_NO_COMPATIBILITY
        void _migrate(cpputils::unique_ref<blobstore::Blob> node, const FsBlobView::Metadata& metadata, FsBlobView::BlobType type, cpputils::SignalCatcher* signalCatcher, const std::function<void(uint32_t numNodes)>& perBlobCallback);
#endif

            cpputils::unique_ref<blobstore::BlobStore> _baseBlobStore;

            DISALLOW_COPY_AND_ASSIGN(FsBlobStore);
        };

        inline FsBlobStore::FsBlobStore(cpputils::unique_ref<blobstore::BlobStore> baseBlobStore)
                : _baseBlobStore(std::move(baseBlobStore)) {
        }

        inline cpputils::unique_ref<FileBlob> FsBlobStore::createFileBlob(const FsBlobView::Metadata& meta) {
            auto blob = _baseBlobStore->create();
            return FileBlob::InitializeEmptyFile(std::move(blob), meta);
        }

        inline cpputils::unique_ref<DirBlob> FsBlobStore::createDirBlob(const FsBlobView::Metadata &meta) {
            auto blob = _baseBlobStore->create();
            return DirBlob::InitializeEmptyDir(std::move(blob), meta);
        }

        inline cpputils::unique_ref<SymlinkBlob> FsBlobStore::createSymlinkBlob(const boost::filesystem::path &target, const FsBlobView::Metadata &meta) {
            auto blob = _baseBlobStore->create();
            return SymlinkBlob::InitializeSymlink(std::move(blob), target, meta);
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

        inline void FsBlobStore::remove(const blockstore::BlockId &blockId) {
            _baseBlobStore->remove(blockId);
        }

        inline uint64_t FsBlobStore::virtualBlocksizeBytes() const {
            return _baseBlobStore->virtualBlocksizeBytes();
        }
    }
}

#endif
