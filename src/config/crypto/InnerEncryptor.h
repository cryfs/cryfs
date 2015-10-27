#pragma once
#ifndef MESSMER_CRYFS_SRC_CONFIG_CRYPTO_INNERENCRYPTOR_H
#define MESSMER_CRYFS_SRC_CONFIG_CRYPTO_INNERENCRYPTOR_H

#include <messmer/cpp-utils/data/Data.h>
#include <messmer/cpp-utils/pointer/unique_ref.h>
#include <boost/optional.hpp>
#include <messmer/cpp-utils/data/Deserializer.h>
#include <messmer/cpp-utils/data/Serializer.h>

namespace cryfs {
    class InnerEncryptor {
    public:
        virtual cpputils::Data encrypt(const cpputils::Data &plaintext) const = 0;
        virtual boost::optional<cpputils::Data> decrypt(const cpputils::Data &plaintext) const = 0;

    protected:
        static void _checkHeader(cpputils::Deserializer *deserializer);
        static void _writeHeader(cpputils::Serializer *serializer);

    private:
        static const std::string HEADER;
    };
}

#endif
