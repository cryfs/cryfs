#pragma once
#ifndef MESSMER_CPPUTILS_CRYPTO_SYMMETRIC_ENCRYPTIONKEY_H_
#define MESSMER_CPPUTILS_CRYPTO_SYMMETRIC_ENCRYPTIONKEY_H_

#include <cpp-utils/data/FixedSizeData.h>
#include <memory>
#include <cpp-utils/system/memory.h>
#include "../cryptopp_byte.h"
#include <cpp-utils/random/RandomGenerator.h>

namespace cpputils {

/**
 * Use this to store an encryption key and keep it safe in memory.
 * It will only keep the key in one memory location, even if the EncryptionKey object is copied or moved.
 * This one memory location will be prevented from swapping to disk.
 * Note: This is a best-effort, but not a guarantee. System hibernation might still write the encryption key to the disk.
 * Also, when (de)serializing the config file or calling a crypto library with the encryption key, it isn't guaranteed
 * that there aren't any copies made to different memory regions. However, these other memory regions should be short-lived
 * and therefore much less likely to swap.
 */
class EncryptionKey final {
private:
    explicit EncryptionKey(std::shared_ptr<Data> keyData)
        : _keyData(std::move(keyData)) {
    }

public:
    EncryptionKey(const EncryptionKey& rhs) = default;
    EncryptionKey(EncryptionKey&& rhs) = default;
    EncryptionKey& operator=(const EncryptionKey& rhs) = default;
    EncryptionKey& operator=(EncryptionKey&& rhs) = default;

    size_t binaryLength() const {
      return _keyData->size();
    }

    size_t stringLength() const {
      return 2 * binaryLength();
    }

    static EncryptionKey Null(size_t keySize) {
        auto data = std::make_shared<Data>(
            keySize,
            make_unique_ref<UnswappableAllocator>()
        );
        data->FillWithZeroes();
        return EncryptionKey(std::move(data));
    }

    static EncryptionKey FromString(const std::string& keyData) {
        auto data = std::make_shared<Data>(
            Data::FromString(keyData, make_unique_ref<UnswappableAllocator>())
        );
        EncryptionKey key(std::move(data));
        ASSERT(key.stringLength() == keyData.size(), "Wrong input size for EncryptionKey::FromString()");

        return key;
    }

    std::string ToString() const {
        auto result = _keyData->ToString();
        ASSERT(result.size() == stringLength(), "Wrong string length");
        return result;
    }

    static EncryptionKey CreateKey(RandomGenerator &randomGenerator, size_t keySize) {
        EncryptionKey result(std::make_shared<Data>(
            keySize,
            make_unique_ref<UnswappableAllocator>() // the allocator makes sure key data is never swapped to disk
        ));
        randomGenerator.write(result._keyData->data(), keySize);
        return result;
    }

    const void *data() const {
        return _keyData->data();
    }

    void *data() {
        return const_cast<void*>(const_cast<const EncryptionKey*>(this)->data());
    }

    // TODO Test take/drop

    EncryptionKey take(size_t numTaken) const {
        ASSERT(numTaken <= _keyData->size(), "Out of bounds");
        auto result = std::make_shared<Data>(numTaken, make_unique_ref<UnswappableAllocator>());
        std::memcpy(result->data(), _keyData->data(), numTaken);
        return EncryptionKey(std::move(result));
    }

    EncryptionKey drop(size_t numDropped) const {
        ASSERT(numDropped <= _keyData->size(), "Out of bounds");
        const size_t resultSize = _keyData->size() - numDropped;
        auto result = std::make_shared<Data>(resultSize, make_unique_ref<UnswappableAllocator>());
        std::memcpy(result->data(), _keyData->dataOffset(numDropped), resultSize);
        return EncryptionKey(std::move(result));
    }

private:
    std::shared_ptr<Data> _keyData;
};

}

#endif
