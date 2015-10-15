#pragma once
#ifndef MESSMER_CRYFS_FILESYSTEM_FSBLOBSTORE_FSBLOB_H
#define MESSMER_CRYFS_FILESYSTEM_FSBLOBSTORE_FSBLOB_H

#include <messmer/cpp-utils/pointer/unique_ref.h>
#include <messmer/blobstore/interface/Blob.h>

namespace cryfs {
    namespace fsblobstore {
        class FsBlob {
        public:
            virtual ~FsBlob();

            virtual off_t lstat_size() const = 0;
            const blockstore::Key &key() const;

        protected:
            FsBlob(cpputils::unique_ref<blobstore::Blob> baseBlob);

            blobstore::Blob &baseBlob();
            const blobstore::Blob &baseBlob() const;

            unsigned char magicNumber() const;
            static unsigned char magicNumber(const blobstore::Blob &blob);

            static void InitializeBlobWithMagicNumber(blobstore::Blob *blob, unsigned char magicNumber);

            friend class FsBlobStore;
            virtual cpputils::unique_ref<blobstore::Blob> releaseBaseBlob();

        private:

            cpputils::unique_ref<blobstore::Blob> _baseBlob;

            DISALLOW_COPY_AND_ASSIGN(FsBlob);
        };


        // ---------------------------
        // Inline function definitions
        // ---------------------------

        inline FsBlob::FsBlob(cpputils::unique_ref<blobstore::Blob> baseBlob)
                : _baseBlob(std::move(baseBlob)) {
        }

        inline FsBlob::~FsBlob() {
        }

        inline const blockstore::Key &FsBlob::key() const {
            return _baseBlob->key();
        }

        inline const blobstore::Blob &FsBlob::baseBlob() const {
            return *_baseBlob;
        }

        inline blobstore::Blob &FsBlob::baseBlob() {
            return *_baseBlob;
        }

        inline unsigned char FsBlob::magicNumber(const blobstore::Blob &blob) {
            unsigned char value;
            blob.read(&value, 0, 1);
            return value;
        }

        inline unsigned char FsBlob::magicNumber() const {
            return magicNumber(*_baseBlob);
        }

        inline void FsBlob::InitializeBlobWithMagicNumber(blobstore::Blob *blob, unsigned char magicNumber) {
            blob->resize(1);
            blob->write(&magicNumber, 0, 1);
        }

        inline cpputils::unique_ref<blobstore::Blob> FsBlob::releaseBaseBlob() {
            return std::move(_baseBlob);
        }
    }
}

#endif
