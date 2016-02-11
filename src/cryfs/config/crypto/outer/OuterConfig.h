#pragma once
#ifndef MESSMER_CRYFS_SRC_CONFIG_CRYPTO_OUTER_OUTERCONFIG_H
#define MESSMER_CRYFS_SRC_CONFIG_CRYPTO_OUTER_OUTERCONFIG_H

#include <cpp-utils/crypto/kdf/DerivedKeyConfig.h>
#include <cpp-utils/data/Data.h>
#include <cpp-utils/data/Serializer.h>
#include <cpp-utils/data/Deserializer.h>

namespace cryfs {
    struct OuterConfig final {
        cpputils::DerivedKeyConfig keyConfig;
        cpputils::Data encryptedInnerConfig;

        cpputils::Data serialize() const;
        static boost::optional<OuterConfig> deserialize(const cpputils::Data &data);

    private:
        static void _checkHeader(cpputils::Deserializer *deserializer);
        static void _writeHeader(cpputils::Serializer *serializer);

        static const std::string HEADER;
    };
}

#endif
