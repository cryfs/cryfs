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

            fspp::num_bytes_t read(void *target, fspp::num_bytes_t offset, fspp::num_bytes_t count) const;

            void write(const void *source, fspp::num_bytes_t offset, fspp::num_bytes_t count);

            void flush();

            void resize(fspp::num_bytes_t size);

            fspp::num_bytes_t lstat_size() const override;

            fspp::num_bytes_t size() const;
        private:
            DISALLOW_COPY_AND_ASSIGN(FileBlob);
        };
    }
}

#endif
