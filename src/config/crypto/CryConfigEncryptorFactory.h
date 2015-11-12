#pragma once
#ifndef MESSMER_CRYFS_SRC_CONFIG_CRYPTO_CRYCONFIGENCRYPTORFACTORY_H
#define MESSMER_CRYFS_SRC_CONFIG_CRYPTO_CRYCONFIGENCRYPTORFACTORY_H

#include "inner/ConcreteInnerEncryptor.h"
#include "CryConfigEncryptor.h"
#include <messmer/cpp-utils/pointer/unique_ref.h>
#include <messmer/cpp-utils/crypto/kdf/Scrypt.h>
#include "../CryCipher.h"

namespace cryfs {
    class CryConfigEncryptorFactory {
    public:
        static cpputils::unique_ref<CryConfigEncryptor> deriveKey(const std::string &password, const cpputils::SCryptSettings &scryptSettings);

        static boost::optional<cpputils::unique_ref<CryConfigEncryptor>> loadKey(const cpputils::Data &ciphertext,
                                                                                 const std::string &password);

    private:

        static cpputils::DerivedKey<CryConfigEncryptor::MaxTotalKeySize> _deriveKey(const cpputils::DerivedKeyConfig &keyConfig, const std::string &password);
        static boost::optional<std::string> _loadCipherName(const OuterEncryptor &outerEncryptor, const OuterConfig &outerConfig);
    };
}

#endif
