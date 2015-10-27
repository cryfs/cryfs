#pragma once
#ifndef MESSMER_CRYFS_SRC_CONFIG_CRYPTO_CONCRETECRYCONFIGENCRYPTOR_H
#define MESSMER_CRYFS_SRC_CONFIG_CRYPTO_CONCRETECRYCONFIGENCRYPTOR_H

#include <messmer/cpp-utils/data/Serializer.h>
#include <messmer/cpp-utils/data/Deserializer.h>
#include "RandomPadding.h"
#include "InnerEncryptor.h"
#include "kdf/DerivedKey.h"

namespace cryfs {
    //TODO Test
    template<class Cipher>
    class ConcreteInnerEncryptor: public InnerEncryptor {
    public:
        static constexpr size_t CONFIG_SIZE = 1024;  // Config data is grown to this size before encryption to hide its actual size

        ConcreteInnerEncryptor(typename Cipher::EncryptionKey key);

        cpputils::Data encrypt(const cpputils::Data &plaintext) const override;
        boost::optional<cpputils::Data> decrypt(const cpputils::Data &ciphertext) const override;
    private:

        typename Cipher::EncryptionKey _key;
    };



    template<class Cipher>
    ConcreteInnerEncryptor<Cipher>::ConcreteInnerEncryptor(typename Cipher::EncryptionKey key): _key(std::move(key)) {
    }

    template<class Cipher>
    boost::optional<cpputils::Data> ConcreteInnerEncryptor<Cipher>::decrypt(const cpputils::Data &ciphertext) const {
        auto decrypted = Cipher::decrypt(static_cast<const uint8_t*>(ciphertext.data()), ciphertext.size(), _key);
        if (decrypted == boost::none) {
            return boost::none;
        }
        auto configData = RandomPadding::remove(*decrypted);
        if (configData == boost::none) {
            return boost::none;
        }
        return std::move(*configData);
    }

    template<class Cipher>
    cpputils::Data ConcreteInnerEncryptor<Cipher>::encrypt(const cpputils::Data &plaintext) const {
        auto paddedPlaintext = RandomPadding::add(plaintext, CONFIG_SIZE);
        return Cipher::encrypt(static_cast<const uint8_t*>(paddedPlaintext.data()), paddedPlaintext.size(), _key);
    }
}

#endif
