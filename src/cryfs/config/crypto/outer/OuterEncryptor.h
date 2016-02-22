#pragma once
#ifndef MESSMER_CRYFS_SRC_CONFIG_CRYPTO_OUTER_OUTERENCRYPTOR_H
#define MESSMER_CRYFS_SRC_CONFIG_CRYPTO_OUTER_OUTERENCRYPTOR_H

#include <cpp-utils/pointer/unique_ref.h>
#include <cpp-utils/data/Deserializer.h>
#include <cpp-utils/data/Serializer.h>
#include <cpp-utils/crypto/symmetric/ciphers.h>
#include "OuterConfig.h"

namespace cryfs {
    class OuterEncryptor final {
    public:
        using Cipher = cpputils::AES256_GCM;
        static constexpr size_t CONFIG_SIZE = 1024;  // Config data is grown to this size before encryption to hide its actual size

        OuterEncryptor(Cipher::EncryptionKey key, cpputils::Data kdfParameters);

        OuterConfig encrypt(const cpputils::Data &encryptedInnerConfig) const;
        boost::optional<cpputils::Data> decrypt(const OuterConfig &outerConfig) const;

    private:

        Cipher::EncryptionKey _key;
        cpputils::Data _kdfParameters;

        DISALLOW_COPY_AND_ASSIGN(OuterEncryptor);
    };
}

#endif
