#pragma once
#ifndef MESSMER_CRYFS_FILESYSTEM_FSBLOBSTORE_FSBLOBVIEW_H
#define MESSMER_CRYFS_FILESYSTEM_FSBLOBSTORE_FSBLOBVIEW_H

#include <blobstore/interface/Blob.h>
#include <cpp-utils/pointer/unique_ref.h>
#include <cpp-utils/system/time.h>
#include <fspp/fs_interface/Types.h>
#include <cryfs/impl/filesystem/fsblobstore/utils/TimestampUpdateBehavior.h>
#include "utils/DirEntry.h"

namespace cryfs {

    //TODO Test
    class FsBlobView final : public blobstore::Blob {
    public:

        //using Metadata = fspp::stat_info;
        struct Metadata  {
          fspp::stat_info _info;

          Metadata()  = default;
          Metadata(uint32_t nlink, fspp::mode_t mode, fspp::uid_t uid, fspp::gid_t gid, fspp::num_bytes_t size, timespec atime, timespec mtime, timespec ctime);

        static Metadata rootMetaData();


        };
        //TODO Rename to "Type" or similar
        using BlobType = fspp::Dir::NodeType;
        using Lock = std::lock_guard<std::mutex>;


        explicit FsBlobView(cpputils::unique_ref<blobstore::Blob> baseBlob, const fsblobstore::TimestampUpdateBehavior behav): _timestampUpdateBehavior(behav), _baseBlob(std::move(baseBlob)), _blobType(BlobType::DIR) {  // blob type overwritten by _loadMetadata, needed for clang-tidy
            _checkHeader(*_baseBlob);
            _loadMetadata();
        }

        // Whoever calls us, we will correctly set the type in our metadata.
        static void InitializeBlob(blobstore::Blob *baseBlob, Metadata metadata, BlobType type);

        static BlobType blobType(const blobstore::Blob &blob) {
            _checkHeader(blob);
            return _getBlobType(blob);
        }

        BlobType blobType() const {
          return _blobType;
        }

        const blockstore::BlockId &blockId() const override {
            return _baseBlob->blockId();
        }

        uint64_t size() const override {
            return _baseBlob->size() - HEADER_SIZE;
        }

        void resize(uint64_t numBytes) override;
        cpputils::Data readAll() const override;
        void read(void *target, uint64_t offset, uint64_t size) const override;
        uint64_t tryRead(void *target, uint64_t offset, uint64_t size) const override;
        void write(const void *source, uint64_t offset, uint64_t size) override;

        void chown(fspp::uid_t uid, fspp::gid_t gid);
        void chmod(fspp::mode_t mode);
        void utimens(timespec atime, timespec mtime);

        // increase link count by one
        void link();
        // decrease link count by one and return true iff this was the last link and the node has
        // to be removed. Not that the removal must be done externally;
        bool unlink();

        fspp::stat_info stat();

        void updateModificationTimestamp();
        void updateAccessTimestamp() const;
        void updateChangeTimestamp();

        void flush() override {
            _storeMetadata();
            return _baseBlob->flush();
        }

        uint32_t numNodes() const override {
            return _baseBlob->numNodes();
        }

        cpputils::unique_ref<blobstore::Blob> releaseBaseBlob() {
            return std::move(_baseBlob);
        }

      const Metadata& metadata() {
        return _metadata;
      }


        //void setMetadata(const Metadata& metadata);
        static uint16_t getFormatVersionHeader(const blobstore::Blob &blob);

#ifndef CRYFS_NO_COMPATIBILITY
        static std::vector<cryfs::fsblobstore::DirEntryWithMetaData> migrate(blobstore::Blob *blob, Metadata metadata, BlobType type);
#endif

    private:
        static constexpr uint16_t FORMAT_VERSION_HEADER = 2;
        static constexpr unsigned int HEADER_SIZE = sizeof(FORMAT_VERSION_HEADER) + sizeof(Metadata);
        constexpr static fspp::num_bytes_t DIR_LSTAT_SIZE = fspp::num_bytes_t(4096);

        void _updateModificationTimestamp();
        void _updateAccessTimestamp() const;
        void _updateChangeTimestamp();

        static void _checkHeader(const blobstore::Blob &blob);

        static BlobType _getBlobType(const blobstore::Blob &blob) {
            Metadata result;
            blob.read(&result, sizeof(FORMAT_VERSION_HEADER), sizeof(Metadata));
            return _metadataToBlobtype(result);
        }

        void _storeMetadata() const {
          _baseBlob->write(&_metadata, sizeof(FORMAT_VERSION_HEADER), sizeof(_metadata));
        }

        void _loadMetadata() {
          _baseBlob->read(&_metadata, sizeof(FORMAT_VERSION_HEADER), sizeof(_metadata));
          _blobType = _metadataToBlobtype(_metadata);

        }


        static BlobType _metadataToBlobtype(const Metadata& metadata);

      const cryfs::fsblobstore::TimestampUpdateBehavior _timestampUpdateBehavior;

        // by having this mutable we can make updateAccessTimePoint const.
        // by the locking used in the public methods all const methods become threadsafe.
        mutable cpputils::unique_ref<blobstore::Blob> _baseBlob;

        mutable Metadata _metadata;
        mutable std::mutex _mutex;

        // this never changes, so we can load it during initialization.
        BlobType _blobType;

        DISALLOW_COPY_AND_ASSIGN(FsBlobView);
    };

}


#endif
