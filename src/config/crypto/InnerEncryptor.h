#pragma once
#ifndef MESSMER_CRYFS_SRC_CONFIG_CRYPTO_INNERENCRYPTOR_H
#define MESSMER_CRYFS_SRC_CONFIG_CRYPTO_INNERENCRYPTOR_H

#include <messmer/cpp-utils/data/Data.h>
#include <messmer/cpp-utils/pointer/unique_ref.h>
#include <boost/optional.hpp>

namespace cryfs {
    class InnerEncryptor {
    public:
        virtual cpputils::Data encrypt(const cpputils::Data &plaintext) const = 0;
        virtual boost::optional <cpputils::Data> decrypt(const cpputils::Data &plaintext) const = 0;
    };
}

#endif
