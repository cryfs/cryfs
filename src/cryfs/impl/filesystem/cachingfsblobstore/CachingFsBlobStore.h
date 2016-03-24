#pragma once
#ifndef MESSMER_CRYFS_FILESYSTEM_CACHINGFSBLOBSTORE_CACHINGFSBLOBSTORE_H
#define MESSMER_CRYFS_FILESYSTEM_CACHINGFSBLOBSTORE_CACHINGFSBLOBSTORE_H

#include <cpp-utils/pointer/unique_ref.h>
#include "../fsblobstore/FsBlobStore.h"
#include <blockstore/implementations/caching/cache/Cache.h>
#include "FileBlobRef.h"
#include "DirBlobRef.h"
#include "SymlinkBlobRef.h"

namespace cryfs {
    namespace cachingfsblobstore {
        //TODO Test classes in cachingfsblobstore

        //TODO Inherit from same interface as FsBlobStore?
        class CachingFsBlobStore final {
        public:
            CachingFsBlobStore(cpputils::unique_ref<fsblobstore::FsBlobStore> baseBlobStore);
            ~CachingFsBlobStore();

            cpputils::unique_ref<FileBlobRef> createFileBlob();
            cpputils::unique_ref<DirBlobRef> createDirBlob();
            cpputils::unique_ref<SymlinkBlobRef> createSymlinkBlob(const boost::filesystem::path &target);
            boost::optional<cpputils::unique_ref<FsBlobRef>> load(const blockstore::Key &key);
            void remove(cpputils::unique_ref<FsBlobRef> blob);
            uint64_t virtualBlocksizeBytes() const;
            uint64_t numBlocks() const;
            uint64_t estimateSpaceForNumBlocksLeft() const;

            void releaseForCache(cpputils::unique_ref<fsblobstore::FsBlob> baseBlob);

        private:
            cpputils::unique_ref<FsBlobRef> _makeRef(cpputils::unique_ref<fsblobstore::FsBlob> baseBlob);

            cpputils::unique_ref<fsblobstore::FsBlobStore> _baseBlobStore;

            //TODO Move Cache to some common location, not in blockstore
            //TODO Use other cache config (i.e. smaller max number of entries) here than in blockstore
            blockstore::caching::Cache<blockstore::Key, cpputils::unique_ref<fsblobstore::FsBlob>, 50> _cache;

            DISALLOW_COPY_AND_ASSIGN(CachingFsBlobStore);
        };


        inline CachingFsBlobStore::CachingFsBlobStore(cpputils::unique_ref<fsblobstore::FsBlobStore> baseBlobStore)
                : _baseBlobStore(std::move(baseBlobStore)), _cache() {
        }

        inline CachingFsBlobStore::~CachingFsBlobStore() {
        }

        inline cpputils::unique_ref<FileBlobRef> CachingFsBlobStore::createFileBlob() {
            // This already creates the file blob in the underlying blobstore.
            // We could also cache this operation, but that is more complicated (blockstore::CachingBlockStore does it)
            // and probably not worth it here.
            return cpputils::make_unique_ref<FileBlobRef>(_baseBlobStore->createFileBlob(), this);
        }

        inline cpputils::unique_ref<DirBlobRef> CachingFsBlobStore::createDirBlob() {
            // This already creates the file blob in the underlying blobstore.
            // We could also cache this operation, but that is more complicated (blockstore::CachingBlockStore does it)
            // and probably not worth it here.
            return cpputils::make_unique_ref<DirBlobRef>(_baseBlobStore->createDirBlob(), this);
        }

        inline cpputils::unique_ref<SymlinkBlobRef> CachingFsBlobStore::createSymlinkBlob(const boost::filesystem::path &target) {
            // This already creates the file blob in the underlying blobstore.
            // We could also cache this operation, but that is more complicated (blockstore::CachingBlockStore does it)
            // and probably not worth it here.
            return cpputils::make_unique_ref<SymlinkBlobRef>(_baseBlobStore->createSymlinkBlob(target), this);
        }

        inline void CachingFsBlobStore::remove(cpputils::unique_ref<FsBlobRef> blob) {
            auto baseBlob = blob->releaseBaseBlob();
            return _baseBlobStore->remove(std::move(baseBlob));
        }

        inline void CachingFsBlobStore::releaseForCache(cpputils::unique_ref<fsblobstore::FsBlob> baseBlob) {
            blockstore::Key key = baseBlob->key();
            _cache.push(key, std::move(baseBlob));
        }

        inline uint64_t CachingFsBlobStore::virtualBlocksizeBytes() const {
            return _baseBlobStore->virtualBlocksizeBytes();
        }

        inline uint64_t CachingFsBlobStore::numBlocks() const {
            return _baseBlobStore->numBlocks();
        }

        inline uint64_t CachingFsBlobStore::estimateSpaceForNumBlocksLeft() const {
            return _baseBlobStore->estimateSpaceForNumBlocksLeft();
        }

    }
}

#endif
