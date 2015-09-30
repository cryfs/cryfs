#ifndef CRYFS_FSBLOBSTORE_FSBLOBSTORE_H
#define CRYFS_FSBLOBSTORE_FSBLOBSTORE_H

#include <messmer/cpp-utils/lock/LockPool.h>
#include <messmer/cpp-utils/pointer/unique_ref.h>
#include <messmer/blobstore/interface/BlobStore.h>
#include "FileBlob.h"
#include "DirBlob.h"
#include "SymlinkBlob.h"

namespace cryfs {
    namespace fsblobstore {
        //TODO Test classes in fsblobstore

        class FsBlobStore {
        public:
            FsBlobStore(cpputils::unique_ref<blobstore::BlobStore> baseBlobStore);

            cpputils::unique_ref<FileBlob> createFileBlob();
            cpputils::unique_ref<DirBlob> createDirBlob();
            cpputils::unique_ref<SymlinkBlob> createSymlinkBlob(const boost::filesystem::path &target);
            boost::optional<cpputils::unique_ref<FsBlob>> load(const blockstore::Key &key);
            void remove(cpputils::unique_ref<FsBlob> blob);

        private:
            std::function<void()> freeLockFunction(const blockstore::Key &key);

            //Instead of locking open blobs, it would be faster to allow parallel access similar to parallelaccessstore.
            cpputils::LockPool<blockstore::Key> _openBlobs;
            cpputils::unique_ref<blobstore::BlobStore> _baseBlobStore;
        };
    }
}

#endif
