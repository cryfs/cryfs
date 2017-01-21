#pragma once
#ifndef MESSMER_CRYFS_SRC_CONFIG_CRYPTO_INNER_INNERENCRYPTOR_H
#define MESSMER_CRYFS_SRC_CONFIG_CRYPTO_INNER_INNERENCRYPTOR_H

#include <cpp-utils/data/Data.h>
#include <cpp-utils/pointer/unique_ref.h>
#include <boost/optional.hpp>
#include <cpp-utils/data/Deserializer.h>
#include <cpp-utils/data/Serializer.h>
#include "InnerConfig.h"

namespace cryfs {
    class InnerEncryptor {
    public:
        virtual ~InnerEncryptor() {}
        virtual InnerConfig encrypt(const cpputils::Data &plaintext) const = 0;
        virtual boost::optional<cpputils::Data> decrypt(const InnerConfig &innerConfig) const = 0;
    };
}

#endif
