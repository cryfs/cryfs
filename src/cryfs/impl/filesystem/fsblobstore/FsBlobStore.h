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
            FsBlobStore(cpputils::unique_ref<blobstore::BlobStore> baseBlobStore);

            cpputils::unique_ref<FileBlob> createFileBlob(const blockstore::BlockId &parent);
            cpputils::unique_ref<DirBlob> createDirBlob(const blockstore::BlockId &parent);
            cpputils::unique_ref<SymlinkBlob> createSymlinkBlob(const boost::filesystem::path &target, const blockstore::BlockId &parent);
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
            void _migrate(cpputils::unique_ref<blobstore::Blob> node, const blockstore::BlockId &parentId, cpputils::SignalCatcher* signalCatcher, std::function<void(uint32_t numNodes)> perBlobCallback);
#endif

            std::function<fspp::num_bytes_t(const blockstore::BlockId &)> _getLstatSize();

            template<class BlobType>
            static cpputils::unique_ref<BlobType> _flushNewBlob(cpputils::unique_ref<BlobType> blob);

            cpputils::unique_ref<blobstore::BlobStore> _baseBlobStore;

            DISALLOW_COPY_AND_ASSIGN(FsBlobStore);
        };

        inline FsBlobStore::FsBlobStore(cpputils::unique_ref<blobstore::BlobStore> baseBlobStore)
                : _baseBlobStore(std::move(baseBlobStore)) {
        }

        template<class BlobType>
        inline cpputils::unique_ref<BlobType> FsBlobStore::_flushNewBlob(cpputils::unique_ref<BlobType> blob) {
            // The caller adds a directory entry for the new blob, which the caches write to disk within a few seconds.
            // Until the new blob is flushed, its header (magic number, parent pointer) only exists in memory, and an open
            // file doesn't flush it until it's closed. A crash in between would leave an entry that points to a blob
            // without a header, which can't be read or deleted. Flushing hands the header to the block cache, which
            // writes it to disk within about a second.
            blob->baseBlob().flush();
            return blob;
        }

        inline cpputils::unique_ref<FileBlob> FsBlobStore::createFileBlob(const blockstore::BlockId &parent) {
            auto blob = _baseBlobStore->create();
            return _flushNewBlob(FileBlob::InitializeEmptyFile(std::move(blob), parent));
        }

        inline cpputils::unique_ref<DirBlob> FsBlobStore::createDirBlob(const blockstore::BlockId &parent) {
            auto blob = _baseBlobStore->create();
            return _flushNewBlob(DirBlob::InitializeEmptyDir(std::move(blob), parent));
        }

        inline cpputils::unique_ref<SymlinkBlob> FsBlobStore::createSymlinkBlob(const boost::filesystem::path &target, const blockstore::BlockId &parent) {
            auto blob = _baseBlobStore->create();
            return _flushNewBlob(SymlinkBlob::InitializeSymlink(std::move(blob), target, parent));
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
