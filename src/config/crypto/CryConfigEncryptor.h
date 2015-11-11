#pragma once
#ifndef MESSMER_CRYFS_SRC_CONFIG_CRYPTO_CRYCONFIGENCRYPTOR_H
#define MESSMER_CRYFS_SRC_CONFIG_CRYPTO_CRYCONFIGENCRYPTOR_H

#include <messmer/cpp-utils/pointer/unique_ref.h>
#include <messmer/cpp-utils/data/Deserializer.h>
#include <messmer/cpp-utils/data/Serializer.h>
#include <messmer/cpp-utils/crypto/kdf/DerivedKeyConfig.h>
#include <messmer/cpp-utils/crypto/kdf/DerivedKey.h>
#include <messmer/cpp-utils/crypto/symmetric/ciphers.h>
#include "inner/InnerEncryptor.h"
#include "outer/OuterEncryptor.h"
#include "../CryCipher.h"

namespace cryfs {
    //TODO Test
    //TODO Test that encrypted config data always has the same size, no matter how big the plaintext config data
    //TODO Test that specified inner cipher is used (e.g. can't be decrypted with other cipher)
    //TODO Use own exception for cpputils::Serializer/cpputils::Deserializer errors and only catch them
    class CryConfigEncryptor {
    public:
        static constexpr size_t OuterKeySize = OuterEncryptor::Cipher::EncryptionKey::BINARY_LENGTH;
        static constexpr size_t MaxTotalKeySize = OuterKeySize + CryCiphers::MAX_KEY_SIZE;

        struct Decrypted {
            cpputils::Data data;
            std::string cipherName;
        };

        CryConfigEncryptor(cpputils::DerivedKey<MaxTotalKeySize> derivedKey);

        cpputils::Data encrypt(const cpputils::Data &plaintext, const std::string &cipherName) const;
        boost::optional<Decrypted> decrypt(const cpputils::Data &data) const;

    private:
        boost::optional<InnerConfig> _loadInnerConfig(const cpputils::Data &data) const;
        cpputils::unique_ref<OuterEncryptor> _outerEncryptor() const;
        cpputils::unique_ref<InnerEncryptor> _innerEncryptor(const std::string &cipherName) const;

        cpputils::DerivedKey<MaxTotalKeySize> _derivedKey;
    };
}

#endif
