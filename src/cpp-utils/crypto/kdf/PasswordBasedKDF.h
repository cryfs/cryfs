#pragma once
#ifndef MESSMER_CPPUTILS_CRYPTO_KDF_PASSWORDBASEDKDF_H
#define MESSMER_CPPUTILS_CRYPTO_KDF_PASSWORDBASEDKDF_H

#include "../../crypto/symmetric/EncryptionKey.h"
#include "../../data/Data.h"

namespace cpputils {

    class PasswordBasedKDF {
    public:
        virtual ~PasswordBasedKDF() = default;

        virtual EncryptionKey deriveKey(size_t keySize, const std::string &password) = 0;
        virtual const Data &kdfParameters() const = 0;
    };

}


#endif
