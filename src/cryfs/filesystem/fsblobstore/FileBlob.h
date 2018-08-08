#pragma once
#ifndef MESSMER_CRYFS_FILESYSTEM_FSBLOBSTORE_FILEBLOB_H_
#define MESSMER_CRYFS_FILESYSTEM_FSBLOBSTORE_FILEBLOB_H_

#include "FsBlob.h"

namespace cryfs {
    namespace fsblobstore {

        class FileBlob final: public FsBlob {
        public:
            static cpputils::unique_ref<FileBlob> InitializeEmptyFile(cpputils::unique_ref<blobstore::Blob> blob, const blockstore::BlockId &parent);

            FileBlob(cpputils::unique_ref<blobstore::Blob> blob);

            size_t read(void *target, uint64_t offset, uint64_t count) const;

            void write(const void *source, uint64_t offset, uint64_t count);

            void flush();

            void resize(off_t size);

            off_t lstat_size() const override;

            off_t size() const;
        private:
            DISALLOW_COPY_AND_ASSIGN(FileBlob);
        };
    }
}

#endif
