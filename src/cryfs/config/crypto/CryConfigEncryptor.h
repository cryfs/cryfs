#pragma once
#ifndef MESSMER_CRYFS_SRC_CONFIG_CRYPTO_CRYCONFIGENCRYPTOR_H
#define MESSMER_CRYFS_SRC_CONFIG_CRYPTO_CRYCONFIGENCRYPTOR_H

#include <cpp-utils/pointer/unique_ref.h>
#include <cpp-utils/data/Deserializer.h>
#include <cpp-utils/data/Serializer.h>
#include <cpp-utils/crypto/symmetric/ciphers.h>
#include "inner/InnerEncryptor.h"
#include "outer/OuterEncryptor.h"
#include "../CryCipher.h"

namespace cryfs {
    //TODO Use own exception for cpputils::Serializer/cpputils::Deserializer errors and only catch them
    class CryConfigEncryptor final {
    public:
        static constexpr size_t OuterKeySize = OuterEncryptor::Cipher::EncryptionKey::BINARY_LENGTH;
        static constexpr size_t MaxTotalKeySize = OuterKeySize + CryCiphers::MAX_KEY_SIZE;

        struct Decrypted {
            cpputils::Data data;
            std::string cipherName;
            bool wasInDeprecatedConfigFormat;
        };

        CryConfigEncryptor(cpputils::FixedSizeData<MaxTotalKeySize> derivedKey, cpputils::Data _kdfParameters);

        cpputils::Data encrypt(const cpputils::Data &plaintext, const std::string &cipherName) const;
        boost::optional<Decrypted> decrypt(const cpputils::Data &data) const;

    private:
        cpputils::unique_ref<OuterEncryptor> _outerEncryptor() const;
        cpputils::unique_ref<InnerEncryptor> _innerEncryptor(const std::string &cipherName) const;

        cpputils::FixedSizeData<MaxTotalKeySize> _derivedKey;
        cpputils::Data _kdfParameters;

        DISALLOW_COPY_AND_ASSIGN(CryConfigEncryptor);
    };
}

#endif
