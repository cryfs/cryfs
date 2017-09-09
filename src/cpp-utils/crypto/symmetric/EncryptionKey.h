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
template<size_t KeySize>
class EncryptionKey final {
private:
    struct EncryptionKeyData final {
    public:
        constexpr static size_t BINARY_LENGTH = FixedSizeData<KeySize>::BINARY_LENGTH;
        constexpr static size_t STRING_LENGTH = FixedSizeData<KeySize>::STRING_LENGTH;

        EncryptionKeyData(const FixedSizeData<KeySize >& keyData);
        ~EncryptionKeyData();

        // Disallow copying and moving
        EncryptionKeyData(const EncryptionKeyData &rhs) = delete;
        EncryptionKeyData(EncryptionKeyData &&rhs) = delete;
        EncryptionKeyData &operator=(const EncryptionKeyData &rhs) = delete;
        EncryptionKeyData &operator=(EncryptionKeyData &&rhs) = delete;

        FixedSizeData<KeySize> key;
        DontSwapMemoryRAII memoryProtector; // this makes sure that the key data isn't swapped to disk
    };

public:
    constexpr static size_t BINARY_LENGTH = EncryptionKeyData::BINARY_LENGTH;
    constexpr static size_t STRING_LENGTH = EncryptionKeyData::STRING_LENGTH;

    EncryptionKey(const FixedSizeData<KeySize>& keyData);

    static EncryptionKey FromBinary(const void *source);
    static EncryptionKey FromString(const std::string& keyData);
    std::string ToString() const;

    static EncryptionKey CreateKey(RandomGenerator &randomGenerator) {
        EncryptionKey result(FixedSizeData<BINARY_LENGTH>::Null());
        randomGenerator.write(result._key->key.data(), BINARY_LENGTH);
        return result;
    }

    const void *data() const {
        return _key->key.data();
    }

    void *data() {
        return const_cast<void*>(const_cast<const EncryptionKey*>(this)->data());
    }

private:

    std::shared_ptr<EncryptionKeyData> _key;
};

template<size_t KeySize> constexpr size_t EncryptionKey<KeySize>::BINARY_LENGTH;
template<size_t KeySize> constexpr size_t EncryptionKey<KeySize>::STRING_LENGTH;

template<size_t KeySize>
inline EncryptionKey<KeySize>::EncryptionKeyData::EncryptionKeyData(const FixedSizeData<KeySize>& keyData)
: key(keyData)
, memoryProtector(&key, sizeof(key)) {
}

template<size_t KeySize>
inline EncryptionKey<KeySize>::EncryptionKeyData::~EncryptionKeyData() {
    // After destruction, the swap-protection is lifted, but we also don't need the key data anymore.
    // Overwrite it with zeroes.
    std::memset(key.data(), 0, KeySize);
}

template<size_t KeySize>
inline EncryptionKey<KeySize>::EncryptionKey(const FixedSizeData<KeySize>& keyData)
: _key(std::make_shared<EncryptionKeyData>(keyData)) {
}

template<size_t KeySize>
EncryptionKey<KeySize> EncryptionKey<KeySize>::FromBinary(const void *source) {
    return EncryptionKey(FixedSizeData<KeySize>::FromBinary(source));
}

template<size_t KeySize>
EncryptionKey<KeySize> EncryptionKey<KeySize>::FromString(const std::string& keyData) {
     return EncryptionKey(FixedSizeData<KeySize>::FromString(keyData));
}

template<size_t KeySize>
std::string EncryptionKey<KeySize>::ToString() const {
    return _key->key.ToString();
}

}

#endif
