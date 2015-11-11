#pragma once
#ifndef MESSMER_CRYFS_SRC_CONFIG_CRYPTO_OUTER_OUTERCONFIG_H
#define MESSMER_CRYFS_SRC_CONFIG_CRYPTO_OUTER_OUTERCONFIG_H

#include <messmer/cpp-utils/crypto/kdf/DerivedKeyConfig.h>
#include <messmer/cpp-utils/data/Data.h>
#include <messmer/cpp-utils/data/Serializer.h>
#include <messmer/cpp-utils/data/Deserializer.h>

namespace cryfs {
    struct OuterConfig {
        cpputils::DerivedKeyConfig keyConfig;
        cpputils::Data encryptedInnerConfig;

        cpputils::Data serialize();
        static boost::optional<OuterConfig> deserialize(const cpputils::Data &data);

    private:
        static void _checkHeader(cpputils::Deserializer *deserializer);
        static void _writeHeader(cpputils::Serializer *serializer);

        static const std::string HEADER;
    };
}

#endif
