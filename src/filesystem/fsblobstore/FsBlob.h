#ifndef CRYFS_FSBLOBSTORE_FSBLOB_H
#define CRYFS_FSBLOBSTORE_FSBLOB_H

#include <messmer/cpp-utils/pointer/unique_ref.h>
#include <messmer/blobstore/interface/Blob.h>
#include <functional>

namespace cryfs {
    namespace fsblobstore {
        class FsBlob {
        public:
            virtual ~FsBlob();

            virtual off_t lstat_size() const = 0;
            blockstore::Key key() const;

        protected:
            FsBlob(cpputils::unique_ref<blobstore::Blob> baseBlob, std::function<void()> onDestruct);

            blobstore::Blob &baseBlob();
            const blobstore::Blob &baseBlob() const;

            unsigned char magicNumber() const;
            static unsigned char magicNumber(const blobstore::Blob &blob);

            static void InitializeBlobWithMagicNumber(blobstore::Blob *blob, unsigned char magicNumber);

        private:
            friend class FsBlobStore;
            cpputils::unique_ref<blobstore::Blob> releaseBaseBlob();

            cpputils::unique_ref<blobstore::Blob> _baseBlob;
            std::function<void()> _onDestruct;

            DISALLOW_COPY_AND_ASSIGN(FsBlob);
        };

        inline FsBlob::FsBlob(cpputils::unique_ref<blobstore::Blob> baseBlob, std::function<void()> onDestruct)
                : _baseBlob(std::move(baseBlob)), _onDestruct(onDestruct) {
        }

        inline FsBlob::~FsBlob() {
            _onDestruct();
        }

        inline blockstore::Key FsBlob::key() const {
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
