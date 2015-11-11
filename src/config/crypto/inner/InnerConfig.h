#pragma once
#ifndef MESSMER_CRYFS_SRC_CONFIG_CRYPTO_INNER_INNERCONFIG_H
#define MESSMER_CRYFS_SRC_CONFIG_CRYPTO_INNER_INNERCONFIG_H

#include <messmer/cpp-utils/data/Data.h>
#include <messmer/cpp-utils/data/Serializer.h>
#include <messmer/cpp-utils/data/Deserializer.h>

namespace cryfs {
    struct InnerConfig {
        std::string cipherName;
        cpputils::Data encryptedConfig;

        cpputils::Data serialize();
        static boost::optional<InnerConfig> deserialize(const cpputils::Data &data);

    private:
        static void _checkHeader(cpputils::Deserializer *deserializer);
        static void _writeHeader(cpputils::Serializer *serializer);

        static const std::string HEADER;
    };
}

#endif
