#pragma once
#ifndef MESSMER_CRYFS_FILESYSTEM_FSBLOBSTORE_FSBLOB_H
#define MESSMER_CRYFS_FILESYSTEM_FSBLOBSTORE_FSBLOB_H

#include <cpp-utils/pointer/unique_ref.h>
#include <blobstore/interface/Blob.h>
#include "FsBlobView.h"

namespace cryfs {
    namespace fsblobstore {
        class FsBlob {
        public:
            virtual ~FsBlob();

            virtual off_t lstat_size() const = 0;
            const blockstore::Key &key() const;
            const blockstore::Key &parentPointer() const;
            void setParentPointer(const blockstore::Key &parentKey);

        protected:
            FsBlob(cpputils::unique_ref<blobstore::Blob> baseBlob);

            FsBlobView &baseBlob();
            const FsBlobView &baseBlob() const;

            static void InitializeBlob(blobstore::Blob *blob, FsBlobView::BlobType magicNumber, const blockstore::Key &parent);

            friend class FsBlobStore;
            virtual cpputils::unique_ref<blobstore::Blob> releaseBaseBlob();

        private:

            FsBlobView _baseBlob;

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
            return _baseBlob.key();
        }

        inline const FsBlobView &FsBlob::baseBlob() const {
            return _baseBlob;
        }

        inline FsBlobView &FsBlob::baseBlob() {
            return _baseBlob;
        }

        inline void FsBlob::InitializeBlob(blobstore::Blob *blob, FsBlobView::BlobType magicNumber, const blockstore::Key &parent) {
            FsBlobView::InitializeBlob(blob, magicNumber, parent);
        }

        inline cpputils::unique_ref<blobstore::Blob> FsBlob::releaseBaseBlob() {
            return _baseBlob.releaseBaseBlob();
        }

        inline const blockstore::Key &FsBlob::parentPointer() const {
            return _baseBlob.parentPointer();
        }

        inline void FsBlob::setParentPointer(const blockstore::Key &parentKey) {
            return _baseBlob.setParentPointer(parentKey);
        }
    }
}

#endif
