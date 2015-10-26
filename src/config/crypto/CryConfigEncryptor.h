#pragma once
#ifndef MESSMER_CRYFS_SRC_CONFIG_CRYPTO_CRYCONFIGENCRYPTOR_H
#define MESSMER_CRYFS_SRC_CONFIG_CRYPTO_CRYCONFIGENCRYPTOR_H

#include <messmer/cpp-utils/data/Data.h>
#include <messmer/cpp-utils/pointer/unique_ref.h>
#include <messmer/cpp-utils/data/Deserializer.h>
#include <messmer/cpp-utils/data/Serializer.h>
#include "kdf/DerivedKey.h"
#include <string>
#include <stdexcept>

namespace cryfs {
    //TODO Test
    //TODO Test that encrypted config data always has the same size, no matter how big the plaintext config data
    //TODO Don't only encrypt with the main cipher, but also use user specified cipher.
    //TODO Use own exception for cpputils::Serializer/cpputils::Deserializer errors and only catch them
    class CryConfigEncryptor {
    public:
        virtual cpputils::Data encrypt(const cpputils::Data &plaintext) = 0;

        virtual boost::optional <cpputils::Data> decrypt(const cpputils::Data &plaintext) = 0;

        static void checkHeader(cpputils::Deserializer *deserializer);
        static void writeHeader(cpputils::Serializer *serializer);

    private:
        template<class Cipher>
        static DerivedKey<Cipher::EncryptionKey::BINARY_LENGTH> _loadKey(cpputils::Deserializer *deserializer,
                                                                         const std::string &password);

        static const std::string HEADER;
    };
}

#endif
