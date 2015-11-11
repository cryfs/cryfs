#pragma once
#ifndef MESSMER_CRYFS_SRC_CONFIG_CRYPTO_CRYCONFIGENCRYPTORFACTORY_H
#define MESSMER_CRYFS_SRC_CONFIG_CRYPTO_CRYCONFIGENCRYPTORFACTORY_H

#include "inner/ConcreteInnerEncryptor.h"
#include "CryConfigEncryptor.h"
#include <messmer/cpp-utils/pointer/unique_ref.h>
#include <messmer/cpp-utils/crypto/kdf/Scrypt.h>
#include "../CryCipher.h"

namespace cryfs {
    //TODO Test
    class CryConfigEncryptorFactory {
    public:
        template<class Cipher>
        static cpputils::unique_ref<CryConfigEncryptor> deriveKey(const std::string &password, const cpputils::SCryptSettings &scryptSettings);

        static boost::optional<cpputils::unique_ref<CryConfigEncryptor>> loadKey(const cpputils::Data &ciphertext,
                                                                                 const std::string &password);

    private:
        static constexpr size_t OuterKeySize = CryConfigEncryptor::OuterCipher::EncryptionKey::BINARY_LENGTH;
        template<class Cipher> static constexpr size_t TotalKeySize();
        static constexpr size_t MaxTotalKeySize = OuterKeySize + CryCiphers::MAX_KEY_SIZE;

        static cpputils::DerivedKey<MaxTotalKeySize> _deriveKey(const cpputils::DerivedKeyConfig &keyConfig, const std::string &password);
    };

    template<class Cipher> constexpr size_t CryConfigEncryptorFactory::TotalKeySize() {
        return OuterKeySize + Cipher::EncryptionKey::BINARY_LENGTH;
    }

    template<class Cipher>
    cpputils::unique_ref<CryConfigEncryptor> CryConfigEncryptorFactory::deriveKey(const std::string &password, const cpputils::SCryptSettings &scryptSettings) {
        //TODO Use _deriveKey(keyConfig, password) instead and get rid of cpputils::SCryptSettings class in favor of cpputils::DerivedKeyConfig
        auto derivedKey = cpputils::SCrypt().generateKey<TotalKeySize<Cipher>()>(password, scryptSettings);
        auto outerKey = derivedKey.key().template take<OuterKeySize>();
        auto innerKey = derivedKey.key().template drop<OuterKeySize>();
        return cpputils::make_unique_ref<CryConfigEncryptor>(
                   cpputils::make_unique_ref<ConcreteInnerEncryptor<Cipher>>(innerKey),
                   outerKey,
                   derivedKey.moveOutConfig()
               );
    }
}

#endif
