#pragma once
#ifndef MESSMER_CRYFS_FILESYSTEM_FSBLOBSTORE_FSBLOBVIEW_H
#define MESSMER_CRYFS_FILESYSTEM_FSBLOBSTORE_FSBLOBVIEW_H

#include <blobstore/interface/Blob.h>
#include <cpp-utils/pointer/unique_ref.h>

namespace cryfs {

    //TODO Test
    class FsBlobView final : public blobstore::Blob {
    public:
        //TODO Rename to "Type" or similar
        enum class BlobType : uint8_t {
            DIR = 0x00,
            FILE = 0x01,
            SYMLINK = 0x02
        };

        FsBlobView(cpputils::unique_ref<blobstore::Blob> baseBlob): _baseBlob(std::move(baseBlob)), _parentPointer(blockstore::BlockId::Null()) {
            _checkHeader(*_baseBlob);
            _loadParentPointer();
        }

        static void InitializeBlob(blobstore::Blob *baseBlob, BlobType blobType, const blockstore::BlockId &parent) {
            baseBlob->resize(HEADER_SIZE);
            baseBlob->write(&FORMAT_VERSION_HEADER, 0, sizeof(FORMAT_VERSION_HEADER));
            uint8_t blobTypeInt = static_cast<uint8_t>(blobType);
            baseBlob->write(&blobTypeInt, sizeof(FORMAT_VERSION_HEADER), sizeof(uint8_t));
            baseBlob->write(parent.data().data(), sizeof(FORMAT_VERSION_HEADER) + sizeof(uint8_t), blockstore::BlockId::BINARY_LENGTH);
            static_assert(HEADER_SIZE == sizeof(FORMAT_VERSION_HEADER) + sizeof(uint8_t) + blockstore::BlockId::BINARY_LENGTH, "If this fails, the header is not initialized correctly in this function.");
        }

        static BlobType blobType(const blobstore::Blob &blob) {
            _checkHeader(blob);
            return _blobType(blob);
        }

        BlobType blobType() const {
            return _blobType(*_baseBlob);
        }

        const blockstore::BlockId &parentPointer() const {
            return _parentPointer;
        }

        void setParentPointer(const blockstore::BlockId &parentId) {
            _parentPointer = parentId;
            _storeParentPointer();
        }

        const blockstore::BlockId &blockId() const override {
            return _baseBlob->blockId();
        }

        uint64_t size() const override {
            return _baseBlob->size() - HEADER_SIZE;
        }

        void resize(uint64_t numBytes) override {
            return _baseBlob->resize(numBytes + HEADER_SIZE);
        }

        cpputils::Data readAll() const override {
            cpputils::Data data = _baseBlob->readAll();
            cpputils::Data dataWithoutHeader(data.size() - HEADER_SIZE);
            //Can we avoid this memcpy? Maybe by having Data::subdata() that returns a reference to the same memory region? Should we?
            std::memcpy(dataWithoutHeader.data(), data.dataOffset(HEADER_SIZE), dataWithoutHeader.size());
            return dataWithoutHeader;
        }

        void read(void *target, uint64_t offset, uint64_t size) const override {
            return _baseBlob->read(target, offset + HEADER_SIZE, size);
        }

        uint64_t tryRead(void *target, uint64_t offset, uint64_t size) const override {
            return _baseBlob->tryRead(target, offset + HEADER_SIZE, size);
        }

        void write(const void *source, uint64_t offset, uint64_t size) override {
            return _baseBlob->write(source, offset + HEADER_SIZE, size);
        }

        void flush() override {
            return _baseBlob->flush();
        }

        uint32_t numNodes() const override {
            return _baseBlob->numNodes();
        }

        cpputils::unique_ref<blobstore::Blob> releaseBaseBlob() {
            return std::move(_baseBlob);
        }

        static uint16_t getFormatVersionHeader(const blobstore::Blob &blob) {
            static_assert(sizeof(uint16_t) == sizeof(FORMAT_VERSION_HEADER), "Wrong type used to read format version header");
            uint16_t actualFormatVersion = 0;
            blob.read(&actualFormatVersion, 0, sizeof(FORMAT_VERSION_HEADER));
            return actualFormatVersion;
        }

#ifndef CRYFS_NO_COMPATIBILITY
        static void migrate(blobstore::Blob *blob, const blockstore::BlockId &parentId);
#endif

    private:
        static constexpr uint16_t FORMAT_VERSION_HEADER = 1;
        static constexpr unsigned int HEADER_SIZE = sizeof(FORMAT_VERSION_HEADER) + sizeof(uint8_t) + blockstore::BlockId::BINARY_LENGTH;

        static void _checkHeader(const blobstore::Blob &blob) {
            uint16_t actualFormatVersion = getFormatVersionHeader(blob);
            if (FORMAT_VERSION_HEADER != actualFormatVersion) {
                throw std::runtime_error("This file system entity has the wrong format. Was it created with a newer version of CryFS?");
            }
        }

        static BlobType _blobType(const blobstore::Blob &blob) {
            uint8_t result = 0;
            blob.read(&result, sizeof(FORMAT_VERSION_HEADER), sizeof(uint8_t));
            return static_cast<BlobType>(result);
        }

        void _loadParentPointer() {
            auto idData = cpputils::FixedSizeData<blockstore::BlockId::BINARY_LENGTH>::Null();
            _baseBlob->read(idData.data(), sizeof(FORMAT_VERSION_HEADER) + sizeof(uint8_t), blockstore::BlockId::BINARY_LENGTH);
            _parentPointer = blockstore::BlockId(idData);
        }

        void _storeParentPointer() {
            _baseBlob->write(_parentPointer.data().data(), sizeof(FORMAT_VERSION_HEADER) + sizeof(uint8_t), blockstore::BlockId::BINARY_LENGTH);
        }


        cpputils::unique_ref<blobstore::Blob> _baseBlob;
        blockstore::BlockId _parentPointer;

        DISALLOW_COPY_AND_ASSIGN(FsBlobView);
    };

}


#endif
