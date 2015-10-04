#ifndef CRYFS_FILESYSTEM_PARALLELACCESSFSBLOBSTORE_PARALLELACCESSFSBLOBSTORE_H
#define CRYFS_FILESYSTEM_PARALLELACCESSFSBLOBSTORE_PARALLELACCESSFSBLOBSTORE_H

#include <messmer/parallelaccessstore/ParallelAccessStore.h>
#include "FileBlobRef.h"
#include "DirBlobRef.h"
#include "SymlinkBlobRef.h"
#include "../fsblobstore/FsBlobStore.h"

namespace cryfs {
    namespace parallelaccessfsblobstore {
        //TODO Test classes in parallelaccessfsblobstore

        class ParallelAccessFsBlobStore {
        public:
            ParallelAccessFsBlobStore(cpputils::unique_ref<fsblobstore::FsBlobStore> baseBlobStore);

            cpputils::unique_ref<FileBlobRef> createFileBlob();
            cpputils::unique_ref<DirBlobRef> createDirBlob();
            cpputils::unique_ref<SymlinkBlobRef> createSymlinkBlob(const boost::filesystem::path &target);
            boost::optional<cpputils::unique_ref<FsBlobRef>> load(const blockstore::Key &key);
            void remove(cpputils::unique_ref<FsBlobRef> blob);

        private:

            cpputils::unique_ref<fsblobstore::FsBlobStore> _baseBlobStore;
            parallelaccessstore::ParallelAccessStore<fsblobstore::FsBlob, FsBlobRef, blockstore::Key> _parallelAccessStore;

            std::function<off_t (const blockstore::Key &)> _getLstatSize();

            DISALLOW_COPY_AND_ASSIGN(ParallelAccessFsBlobStore);
        };
    }
}

#endif
