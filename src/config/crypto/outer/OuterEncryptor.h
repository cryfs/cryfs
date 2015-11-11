#pragma once
#ifndef MESSMER_CRYFS_SRC_CONFIG_CRYPTO_OUTER_OUTERENCRYPTOR_H
#define MESSMER_CRYFS_SRC_CONFIG_CRYPTO_OUTER_OUTERENCRYPTOR_H

#include <messmer/cpp-utils/pointer/unique_ref.h>
#include <messmer/cpp-utils/data/Deserializer.h>
#include <messmer/cpp-utils/data/Serializer.h>
#include <messmer/cpp-utils/crypto/symmetric/ciphers.h>
#include "OuterConfig.h"

namespace cryfs {
    class OuterEncryptor {
    public:
        using Cipher = cpputils::AES256_GCM;
        static constexpr size_t CONFIG_SIZE = 1024;  // Config data is grown to this size before encryption to hide its actual size

        OuterEncryptor(Cipher::EncryptionKey key, const cpputils::DerivedKeyConfig &keyConfig);

        OuterConfig encrypt(const cpputils::Data &encryptedInnerConfig) const;
        boost::optional<cpputils::Data> decrypt(const OuterConfig &outerConfig) const;

    private:

        Cipher::EncryptionKey _key;
        cpputils::DerivedKeyConfig _keyConfig;
    };
}

#endif
