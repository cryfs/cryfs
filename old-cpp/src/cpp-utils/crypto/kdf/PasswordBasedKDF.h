#pragma once
#ifndef MESSMER_CPPUTILS_CRYPTO_KDF_PASSWORDBASEDKDF_H
#define MESSMER_CPPUTILS_CRYPTO_KDF_PASSWORDBASEDKDF_H

#include "../../crypto/symmetric/EncryptionKey.h"
#include "../../data/Data.h"

namespace cpputils {

    class PasswordBasedKDF {
    public:
        virtual ~PasswordBasedKDF() = default;

        struct KeyResult final {
          cpputils::EncryptionKey key;
          cpputils::Data kdfParameters;
        };

        virtual EncryptionKey deriveExistingKey(size_t keySize, const std::string& password, const Data& kdfParameters) = 0;
        virtual KeyResult deriveNewKey(size_t keySize, const std::string& password) = 0;
    };

}


#endif
