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
        static cpputils::unique_ref<CryConfigEncryptor> deriveKey(const std::string &password);

        static boost::optional <cpputils::unique_ref<CryConfigEncryptor>> loadKey(const cpputils::Data &ciphertext,
                                                                                  const std::string &password);

    private:
        template<class Cipher>
        static DerivedKey<Cipher::EncryptionKey::BINARY_LENGTH> _loadKey(cpputils::Deserializer *deserializer,
                                                                         const std::string &password);
    };

    template<class Cipher, class SCryptConfig>
    cpputils::unique_ref<CryConfigEncryptor> CryConfigEncryptorFactory::deriveKey(const std::string &password) {
        auto key = SCrypt().generateKey<Cipher::EncryptionKey::BINARY_LENGTH, SCryptConfig>(password);
        return cpputils::make_unique_ref<CryConfigEncryptor>(
                   cpputils::make_unique_ref<ConcreteInnerEncryptor<Cipher>>(key.moveOutKey()),
                   key.moveOutConfig()
               );
    }
}

#endif
