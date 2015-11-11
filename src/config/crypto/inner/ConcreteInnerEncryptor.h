#pragma once
#ifndef MESSMER_CRYFS_SRC_CONFIG_CRYPTO_INNER_CONCRETECRYCONFIGENCRYPTOR_H
#define MESSMER_CRYFS_SRC_CONFIG_CRYPTO_INNER_CONCRETECRYCONFIGENCRYPTOR_H

#include <messmer/cpp-utils/crypto/RandomPadding.h>
#include <messmer/cpp-utils/crypto/kdf/DerivedKey.h>

#include "InnerEncryptor.h"
#include "InnerConfig.h"

namespace cryfs {
    //TODO Test
    template<class Cipher>
    class ConcreteInnerEncryptor: public InnerEncryptor {
    public:
        static constexpr size_t CONFIG_SIZE = 512;  // Inner config data is grown to this size before encryption to hide its actual size

        ConcreteInnerEncryptor(typename Cipher::EncryptionKey key);

        cpputils::Data encrypt(const cpputils::Data &plaintext) const override;
        boost::optional<cpputils::Data> decrypt(const cpputils::Data &ciphertext) const override;
    private:

        typename Cipher::EncryptionKey _key;
    };

    template<class Cipher>
    ConcreteInnerEncryptor<Cipher>::ConcreteInnerEncryptor(typename Cipher::EncryptionKey key)
            : _key(std::move(key)) {
    }

    template<class Cipher>
    boost::optional<cpputils::Data> ConcreteInnerEncryptor<Cipher>::decrypt(const cpputils::Data &ciphertext) const {
        auto innerConfig = InnerConfig::deserialize(ciphertext);
        if (innerConfig == boost::none) {
            return boost::none;
        }
        if (innerConfig->cipherName != Cipher::NAME) {
            cpputils::logging::LOG(cpputils::logging::ERROR) << "Wrong inner cipher used";
            return boost::none;
        }
        auto decrypted = Cipher::decrypt(static_cast<const uint8_t*>(innerConfig->encryptedConfig.data()), innerConfig->encryptedConfig.size(), _key);
        if (decrypted == boost::none) {
            return boost::none;
        }
        auto configData = cpputils::RandomPadding::remove(*decrypted);
        if (configData == boost::none) {
            return boost::none;
        }
        return std::move(*configData);
    }

    template<class Cipher>
    cpputils::Data ConcreteInnerEncryptor<Cipher>::encrypt(const cpputils::Data &plaintext) const {
        auto paddedPlaintext = cpputils::RandomPadding::add(plaintext, CONFIG_SIZE);
        auto encrypted = Cipher::encrypt(static_cast<const uint8_t*>(paddedPlaintext.data()), paddedPlaintext.size(), _key);
        return InnerConfig{Cipher::NAME, std::move(encrypted)}.serialize();
    }
}

#endif
