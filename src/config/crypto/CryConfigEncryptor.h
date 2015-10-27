#pragma once
#ifndef MESSMER_CRYFS_SRC_CONFIG_CRYPTO_CRYCONFIGENCRYPTOR_H
#define MESSMER_CRYFS_SRC_CONFIG_CRYPTO_CRYCONFIGENCRYPTOR_H

#include <messmer/cpp-utils/pointer/unique_ref.h>
#include <messmer/cpp-utils/data/Deserializer.h>
#include <messmer/cpp-utils/data/Serializer.h>
#include "InnerEncryptor.h"
#include "kdf/DerivedKeyConfig.h"

namespace cryfs {
    //TODO Test
    //TODO Test that encrypted config data always has the same size, no matter how big the plaintext config data
    //TODO Don't only encrypt with the main cipher, but also use user specified cipher.
    //TODO Use own exception for cpputils::Serializer/cpputils::Deserializer errors and only catch them
    class CryConfigEncryptor {
    public:
        CryConfigEncryptor(cpputils::unique_ref<InnerEncryptor> innerEncryptor, DerivedKeyConfig keyConfig);

        cpputils::Data encrypt(const cpputils::Data &plaintext);
        boost::optional <cpputils::Data> decrypt(const cpputils::Data &plaintext);

        static void checkHeader(cpputils::Deserializer *deserializer);
        static void writeHeader(cpputils::Serializer *serializer);

    private:
        void _ignoreKey(cpputils::Deserializer *deserializer);
        boost::optional<cpputils::Data> _loadAndDecryptConfigData(cpputils::Deserializer *deserializer);
        cpputils::Data _serialize(const cpputils::Data &ciphertext);

        cpputils::unique_ref<InnerEncryptor> _innerEncryptor;
        DerivedKeyConfig _keyConfig;

        static const std::string HEADER;
    };
}

#endif
