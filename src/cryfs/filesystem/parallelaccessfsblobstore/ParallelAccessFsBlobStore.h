#pragma once
#ifndef MESSMER_CRYFS_FILESYSTEM_PARALLELACCESSFSBLOBSTORE_PARALLELACCESSFSBLOBSTORE_H
#define MESSMER_CRYFS_FILESYSTEM_PARALLELACCESSFSBLOBSTORE_PARALLELACCESSFSBLOBSTORE_H

#include <parallelaccessstore/ParallelAccessStore.h>
#include "FileBlobRef.h"
#include "DirBlobRef.h"
#include "SymlinkBlobRef.h"
#include "../cachingfsblobstore/CachingFsBlobStore.h"
#include "ParallelAccessFsBlobStoreAdapter.h"

namespace cryfs {
    namespace parallelaccessfsblobstore {
        //TODO Test classes in parallelaccessfsblobstore

        //TODO Race condition: Thread 1 destructs CachingFsBlobStore element from ParallelAccessFsBlobStore, but
        //                     it didn't get written into cache yet, when Thread 2 requests it.
        //                     Same race condition in Caching/ParallelAccessBlockStore?

        class ParallelAccessFsBlobStore final {
        public:
            ParallelAccessFsBlobStore(cpputils::unique_ref<cachingfsblobstore::CachingFsBlobStore> baseBlobStore);

            cpputils::unique_ref<FileBlobRef> createFileBlob();
            cpputils::unique_ref<DirBlobRef> createDirBlob();
            cpputils::unique_ref<SymlinkBlobRef> createSymlinkBlob(const boost::filesystem::path &target);
            boost::optional<cpputils::unique_ref<FsBlobRef>> load(const blockstore::Key &key);
            void remove(cpputils::unique_ref<FsBlobRef> blob);

        private:

            cpputils::unique_ref<cachingfsblobstore::CachingFsBlobStore> _baseBlobStore;
            parallelaccessstore::ParallelAccessStore<cachingfsblobstore::FsBlobRef, FsBlobRef, blockstore::Key> _parallelAccessStore;

            std::function<off_t (const blockstore::Key &)> _getLstatSize();

            DISALLOW_COPY_AND_ASSIGN(ParallelAccessFsBlobStore);
        };

        inline ParallelAccessFsBlobStore::ParallelAccessFsBlobStore(cpputils::unique_ref<cachingfsblobstore::CachingFsBlobStore> baseBlobStore)
                : _baseBlobStore(std::move(baseBlobStore)),
                  _parallelAccessStore(cpputils::make_unique_ref<ParallelAccessFsBlobStoreAdapter>(_baseBlobStore.get())) {
        }

        void ParallelAccessFsBlobStore::remove(cpputils::unique_ref<FsBlobRef> blob) {
            blockstore::Key key = blob->key();
            return _parallelAccessStore.remove(key, std::move(blob));
        }

        std::function<off_t (const blockstore::Key &key)> ParallelAccessFsBlobStore::_getLstatSize() {
            return [this] (const blockstore::Key &key) {
                auto blob = load(key);
                ASSERT(blob != boost::none, "Blob not found");
                return (*blob)->lstat_size();
            };
        }
        
    }
}

#endif
