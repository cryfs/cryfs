#pragma once
#ifndef MESSMER_CRYFS_FILESYSTEM_PARALLELACCESSFSBLOBSTORE_PARALLELACCESSFSBLOBSTORE_H
#define MESSMER_CRYFS_FILESYSTEM_PARALLELACCESSFSBLOBSTORE_PARALLELACCESSFSBLOBSTORE_H

#include <parallelaccessstore/ParallelAccessStore.h>
#include "FileBlobRef.h"
#include "DirBlobRef.h"
#include "SymlinkBlobRef.h"
#include "cryfs/impl/filesystem/cachingfsblobstore/CachingFsBlobStore.h"
#include "ParallelAccessFsBlobStoreAdapter.h"

namespace cryfs {
    namespace parallelaccessfsblobstore {
        //TODO Test classes in parallelaccessfsblobstore

        //TODO Race condition: Thread 1 destructs CachingFsBlobStore element from ParallelAccessFsBlobStore, but
        //                     it didn't get written into cache yet, when Thread 2 requests it.
        //                     Same race condition in Caching/ParallelAccessBlockStore?

        class ParallelAccessFsBlobStore final {
        public:
            explicit ParallelAccessFsBlobStore(cpputils::unique_ref<cachingfsblobstore::CachingFsBlobStore> baseBlobStore);

            cpputils::unique_ref<FileBlobRef> createFileBlob(const FsBlobView::Metadata &meta);
            cpputils::unique_ref<DirBlobRef> createDirBlob(const FsBlobView::Metadata &meta);
            cpputils::unique_ref<SymlinkBlobRef> createSymlinkBlob(const boost::filesystem::path &target, const FsBlobView::Metadata &);
            boost::optional<cpputils::unique_ref<FsBlobRef>> load(const blockstore::BlockId &blockId);
            void remove(cpputils::unique_ref<FsBlobRef> blob);
            uint64_t virtualBlocksizeBytes() const;
            uint64_t numBlocks() const;
            uint64_t estimateSpaceForNumBlocksLeft() const;

        private:

            cpputils::unique_ref<cachingfsblobstore::CachingFsBlobStore> _baseBlobStore;
            parallelaccessstore::ParallelAccessStore<cachingfsblobstore::FsBlobRef, FsBlobRef, blockstore::BlockId> _parallelAccessStore;

          DISALLOW_COPY_AND_ASSIGN(ParallelAccessFsBlobStore);
        };

        inline ParallelAccessFsBlobStore::ParallelAccessFsBlobStore(cpputils::unique_ref<cachingfsblobstore::CachingFsBlobStore> baseBlobStore)
                : _baseBlobStore(std::move(baseBlobStore)),
                  _parallelAccessStore(cpputils::make_unique_ref<ParallelAccessFsBlobStoreAdapter>(_baseBlobStore.get())) {
        }

        inline void ParallelAccessFsBlobStore::remove(cpputils::unique_ref<FsBlobRef> blob) {
            blockstore::BlockId blockId = blob->blockId();
            return _parallelAccessStore.remove(blockId, std::move(blob));
        }

    inline uint64_t ParallelAccessFsBlobStore::virtualBlocksizeBytes() const {
            return _baseBlobStore->virtualBlocksizeBytes();
        }

        inline uint64_t ParallelAccessFsBlobStore::numBlocks() const {
            return _baseBlobStore->numBlocks();
        }

        inline uint64_t ParallelAccessFsBlobStore::estimateSpaceForNumBlocksLeft() const {
            return _baseBlobStore->estimateSpaceForNumBlocksLeft();
        }
    }
}

#endif
