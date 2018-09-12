#pragma once
#ifndef MESSMER_CPPUTILS_CRYPTO_KDF_PASSWORDBASEDKDF_H
#define MESSMER_CPPUTILS_CRYPTO_KDF_PASSWORDBASEDKDF_H

#include "../../crypto/symmetric/EncryptionKey.h"
#include "../../data/Data.h"

namespace cpputils {

    class PasswordBasedKDF {
    public:
        virtual ~PasswordBasedKDF() {}

        template<size_t KEYSIZE> EncryptionKey<KEYSIZE> deriveKey(const std::string &password);
        virtual const Data &kdfParameters() const = 0;

    protected:
        virtual void derive(void *destination, size_t size, const std::string &password) = 0;
    };

    template<size_t KEYSIZE> EncryptionKey<KEYSIZE>
    inline PasswordBasedKDF::deriveKey(const std::string &password) {
        auto result = EncryptionKey<KEYSIZE>::Null();
        derive(result.data(), result.BINARY_LENGTH, password);
        return result;
    }

}


#endif
