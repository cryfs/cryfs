#pragma once
#ifndef MESSMER_CRYFS_FILESYSTEM_FSBLOBSTORE_FSBLOB_H
#define MESSMER_CRYFS_FILESYSTEM_FSBLOBSTORE_FSBLOB_H

#include <cpp-utils/pointer/unique_ref.h>
#include <blobstore/interface/Blob.h>
#include "FsBlobView.h"
#include <fspp/fs_interface/Types.h>

namespace cryfs {
    namespace fsblobstore {
        class FsBlob {
        public:
            virtual ~FsBlob();

            const blockstore::BlockId &blockId() const;

            const FsBlobView::Metadata& metaData() {
              return baseBlob().metadata();
            }

          void chown(fspp::uid_t uid, fspp::gid_t gid) {
            return baseBlob().chown(uid, gid);
          }

          void chmod(fspp::mode_t mode) {
            return baseBlob().chmod(mode);
          }

          fspp::stat_info stat() {
            return baseBlob().stat();
          }

          // increase link count by one
          void link() {
            return baseBlob().link();
          }
          // decrease link count by one and return if this was the last link and the node has
          // to be removed. Not that the removal must be done externally;
          bool unlink() {
            return baseBlob().unlink();
          }

            void updateAccessTimestamp() const {
              return baseBlob().updateAccessTimestamp();
            }

            void updateModificationTimestamp() {
              return baseBlob().updateModificationTimestamp();
            }

            void updateChangeTimestamp() {
              return baseBlob().updateChangeTimestamp();
            }

        protected:
            FsBlob(cpputils::unique_ref<blobstore::Blob> baseBlob, const fsblobstore::TimestampUpdateBehavior& behavior);

            FsBlobView &baseBlob();
            const FsBlobView &baseBlob() const;

            static void InitializeBlob(blobstore::Blob *blob, const FsBlobView::Metadata& meta, FsBlobView::BlobType type);

            friend class FsBlobStore;
            virtual cpputils::unique_ref<blobstore::Blob> releaseBaseBlob();

        private:

            FsBlobView _baseBlob;

            DISALLOW_COPY_AND_ASSIGN(FsBlob);
        };


        // ---------------------------
        // Inline function definitions
        // ---------------------------

        inline FsBlob::FsBlob(cpputils::unique_ref<blobstore::Blob> baseBlob, const fsblobstore::TimestampUpdateBehavior& behavior)
                : _baseBlob(std::move(baseBlob), behavior) {
        }

        inline FsBlob::~FsBlob() = default;

        inline const blockstore::BlockId &FsBlob::blockId() const {
            return _baseBlob.blockId();
        }

        inline const FsBlobView &FsBlob::baseBlob() const {
            return _baseBlob;
        }

        inline FsBlobView &FsBlob::baseBlob() {
            return _baseBlob;
        }

        inline void FsBlob::InitializeBlob(blobstore::Blob *blob, const FsBlobView::Metadata& metadata, FsBlobView::BlobType type) {
            FsBlobView::InitializeBlob(blob, metadata, type);
        }

        inline cpputils::unique_ref<blobstore::Blob> FsBlob::releaseBaseBlob() {
            return _baseBlob.releaseBaseBlob();
        }

    }
}

#endif
