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

            void releaseForCache(cpputils::unique_ref<fsblobstore::FsBlob> baseBlob);

        private:
            cpputils::unique_ref<FsBlobRef> _makeRef(cpputils::unique_ref<fsblobstore::FsBlob> baseBlob);

            cpputils::unique_ref<fsblobstore::FsBlobStore> _baseBlobStore;

            //TODO Move Cache to some common location, not in blockstore
            //TODO Use other cache config (i.e. smaller max number of entries) here than in blockstore
            blockstore::caching::Cache<blockstore::Key, cpputils::unique_ref<fsblobstore::FsBlob>, 50> _cache;

            DISALLOW_COPY_AND_ASSIGN(CachingFsBlobStore);
        };
    }
}

#endif
