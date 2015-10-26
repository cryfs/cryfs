#pragma once
#ifndef MESSMER_CRYFS_SRC_CONFIG_CRYPTO_CRYCONFIGENCRYPTORFACTORY_H
#define MESSMER_CRYFS_SRC_CONFIG_CRYPTO_CRYCONFIGENCRYPTORFACTORY_H

#include "ConcreteCryConfigEncryptor.h"
#include <messmer/cpp-utils/pointer/unique_ref.h>
#include "kdf/Scrypt.h"

namespace cryfs {
    //TODO Test
    class CryConfigEncryptorFactory {
    public:
        template<class Cipher>
        static cpputils::unique_ref <CryConfigEncryptor> deriveKey(const std::string &password);

        static boost::optional <cpputils::unique_ref<CryConfigEncryptor>> loadKey(const cpputils::Data &ciphertext,
                                                                                  const std::string &password);

    private:
        template<class Cipher>
        static DerivedKey<Cipher::EncryptionKey::BINARY_LENGTH> _loadKey(cpputils::Deserializer *deserializer,
                                                                         const std::string &password);
    };

    template<class Cipher>
    cpputils::unique_ref<CryConfigEncryptor> CryConfigEncryptorFactory::deriveKey(const std::string &password) {
        //TODO This is only kept here to recognize when this is run in tests. After tests are faster, replace this with something in main(), saying something like "Loading configuration file..."
        std::cout << "Deriving secure key for config file..." << std::flush;
        auto key = SCrypt().generateKey<Cipher::EncryptionKey::BINARY_LENGTH>(password);
        std::cout << "done" << std::endl;
        return cpputils::make_unique_ref<ConcreteCryConfigEncryptor<Cipher>>(std::move(key));
    }
}

#endif
