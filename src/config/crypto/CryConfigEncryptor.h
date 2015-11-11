#pragma once
#ifndef MESSMER_CRYFS_SRC_CONFIG_CRYPTO_CRYCONFIGENCRYPTOR_H
#define MESSMER_CRYFS_SRC_CONFIG_CRYPTO_CRYCONFIGENCRYPTOR_H

#include <messmer/cpp-utils/pointer/unique_ref.h>
#include <messmer/cpp-utils/data/Deserializer.h>
#include <messmer/cpp-utils/data/Serializer.h>
#include "inner/InnerEncryptor.h"
#include <messmer/cpp-utils/crypto/kdf/DerivedKeyConfig.h>
#include <messmer/cpp-utils/crypto/symmetric/ciphers.h>

namespace cryfs {
    //TODO Test
    //TODO Test that encrypted config data always has the same size, no matter how big the plaintext config data
    //TODO Use own exception for cpputils::Serializer/cpputils::Deserializer errors and only catch them
    class CryConfigEncryptor {
    public:
        using OuterCipher = cpputils::AES256_GCM;
        static constexpr size_t CONFIG_SIZE = 1024;  // Config data is grown to this size before encryption to hide its actual size

        CryConfigEncryptor(cpputils::unique_ref<InnerEncryptor> innerEncryptor, OuterCipher::EncryptionKey outerKey, cpputils::DerivedKeyConfig keyConfig);

        cpputils::Data encrypt(const cpputils::Data &plaintext);
        boost::optional <cpputils::Data> decrypt(const cpputils::Data &data);

    private:
        boost::optional<cpputils::Data> _decryptInnerConfig(const cpputils::Data &encryptedInnerConfig);

        cpputils::unique_ref<InnerEncryptor> _innerEncryptor;
        OuterCipher::EncryptionKey _outerKey;
        cpputils::DerivedKeyConfig _keyConfig;
    };
}

#endif
