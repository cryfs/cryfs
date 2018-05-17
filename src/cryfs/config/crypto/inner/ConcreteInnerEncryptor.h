#pragma once
#ifndef MESSMER_CRYFS_SRC_CONFIG_CRYPTO_INNER_CONCRETECRYCONFIGENCRYPTOR_H
#define MESSMER_CRYFS_SRC_CONFIG_CRYPTO_INNER_CONCRETECRYCONFIGENCRYPTOR_H

#include <cpp-utils/crypto/RandomPadding.h>

#include "InnerEncryptor.h"
#include "InnerConfig.h"

namespace cryfs {
    template<class Cipher>
    class ConcreteInnerEncryptor final: public InnerEncryptor {
    public:
        static constexpr size_t CONFIG_SIZE = 900;  // Inner config data is grown to this size before encryption to hide its actual size

        ConcreteInnerEncryptor(const typename Cipher::EncryptionKey& key);

        InnerConfig encrypt(const cpputils::Data &config) const override;
        boost::optional<cpputils::Data> decrypt(const InnerConfig &innerConfig) const override;

    private:

        typename Cipher::EncryptionKey _key;

        DISALLOW_COPY_AND_ASSIGN(ConcreteInnerEncryptor);
    };

    template<class Cipher>
    ConcreteInnerEncryptor<Cipher>::ConcreteInnerEncryptor(const typename Cipher::EncryptionKey& key)
            : _key(key) {
    }

    template<class Cipher>
    boost::optional<cpputils::Data> ConcreteInnerEncryptor<Cipher>::decrypt(const InnerConfig &innerConfig) const {
        if (innerConfig.cipherName != Cipher::NAME) {
            cpputils::logging::LOG(cpputils::logging::ERR, "Initialized ConcreteInnerEncryptor with wrong cipher");
            return boost::none;
        }
        auto decrypted = Cipher::decrypt(static_cast<const uint8_t*>(innerConfig.encryptedConfig.data()), innerConfig.encryptedConfig.size(), _key);
        if (decrypted == boost::none) {
            cpputils::logging::LOG(cpputils::logging::ERR, "Failed decrypting configuration file");
            return boost::none;
        }
        auto configData = cpputils::RandomPadding::remove(*decrypted);
        if (configData == boost::none) {
            return boost::none;
        }
        return std::move(*configData);
    }

    template<class Cipher>
    InnerConfig ConcreteInnerEncryptor<Cipher>::encrypt(const cpputils::Data &config) const {
        auto padded = cpputils::RandomPadding::add(config, CONFIG_SIZE);
        auto encrypted = Cipher::encrypt(static_cast<const uint8_t*>(padded.data()), padded.size(), _key);
        return InnerConfig{Cipher::NAME, std::move(encrypted)};
    }
}

#endif
