#pragma once
#ifndef MESSMER_CRYFS_SRC_CONFIG_CRYPTO_CRYCONFIGENCRYPTORFACTORY_H
#define MESSMER_CRYFS_SRC_CONFIG_CRYPTO_CRYCONFIGENCRYPTORFACTORY_H

#include "ConcreteInnerEncryptor.h"
#include "CryConfigEncryptor.h"
#include <messmer/cpp-utils/pointer/unique_ref.h>
#include "kdf/Scrypt.h"

namespace cryfs {
    //TODO Test
    class CryConfigEncryptorFactory {
    public:
        template<class Cipher, class SCryptConfig>
        static cpputils::unique_ref<CryConfigEncryptor> deriveKey(const std::string &password, const std::string &cipherName);

        static boost::optional <cpputils::unique_ref<CryConfigEncryptor>> loadKey(const cpputils::Data &ciphertext,
                                                                                  const std::string &password);

    private:
        static constexpr size_t OuterKeySize = CryConfigEncryptor::OuterCipher::EncryptionKey::BINARY_LENGTH;
        template<class Cipher> static constexpr size_t TotalKeySize();

        template<class Cipher>
        static DerivedKey<CryConfigEncryptor::OuterCipher::EncryptionKey::BINARY_LENGTH + Cipher::EncryptionKey::BINARY_LENGTH>
                _loadKey(cpputils::Deserializer *deserializer, const std::string &password);
    };

    template<class Cipher> constexpr size_t CryConfigEncryptorFactory::TotalKeySize() {
        return OuterKeySize + Cipher::EncryptionKey::BINARY_LENGTH;
    }

    template<class Cipher, class SCryptConfig>
    cpputils::unique_ref<CryConfigEncryptor> CryConfigEncryptorFactory::deriveKey(const std::string &password, const std::string &cipherName) {
        auto derivedKey = SCrypt().generateKey<TotalKeySize<Cipher>(), SCryptConfig>(password);
        auto outerKey = derivedKey.key().template take<OuterKeySize>();
        auto innerKey = derivedKey.key().template drop<OuterKeySize>();
        return cpputils::make_unique_ref<CryConfigEncryptor>(
                   cpputils::make_unique_ref<ConcreteInnerEncryptor<Cipher>>(innerKey, cipherName),
                   outerKey,
                   derivedKey.moveOutConfig()
               );
    }

    template<class Cipher>
    DerivedKey<CryConfigEncryptor::OuterCipher::EncryptionKey::BINARY_LENGTH + Cipher::EncryptionKey::BINARY_LENGTH>
    CryConfigEncryptorFactory::_loadKey(cpputils::Deserializer *deserializer, const std::string &password) {
        auto keyConfig = DerivedKeyConfig::load(deserializer);
        //TODO This is only kept here to recognize when this is run in tests. After tests are faster, replace this with something in main(), saying something like "Loading configuration file..."
        std::cout << "Deriving secure key for config file..." << std::flush;
        auto key = SCrypt().generateKeyFromConfig<TotalKeySize<Cipher>()>(password, keyConfig);
        std::cout << "done" << std::endl;
        return DerivedKey<TotalKeySize<Cipher>()>(std::move(keyConfig), std::move(key));
    }
}

#endif
