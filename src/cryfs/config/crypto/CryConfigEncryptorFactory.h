#pragma once
#ifndef MESSMER_CRYFS_SRC_CONFIG_CRYPTO_CRYCONFIGENCRYPTORFACTORY_H
#define MESSMER_CRYFS_SRC_CONFIG_CRYPTO_CRYCONFIGENCRYPTORFACTORY_H

#include "inner/ConcreteInnerEncryptor.h"
#include "CryConfigEncryptor.h"
#include <cpp-utils/pointer/unique_ref.h>
#include <cpp-utils/crypto/kdf/Scrypt.h>
#include "../CryCipher.h"

namespace cryfs {
    class CryConfigEncryptorFactory final {
    public:
        static cpputils::unique_ref<CryConfigEncryptor> deriveKey(const std::string &password, const cpputils::SCryptSettings &scryptSettings);

        static boost::optional<cpputils::unique_ref<CryConfigEncryptor>> loadKey(const cpputils::Data &ciphertext,
                                                                                 const std::string &password);

    private:

        static cpputils::unique_ref<CryConfigEncryptor> _deriveKey(cpputils::unique_ref<cpputils::SCrypt> kdf, const std::string &password);
    };
}

#endif
