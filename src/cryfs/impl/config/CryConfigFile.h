#pragma once
#ifndef MESSMER_CRYFS_SRC_CONFIG_CRYCONFIGFILE_H
#define MESSMER_CRYFS_SRC_CONFIG_CRYCONFIGFILE_H

#include <boost/optional.hpp>
#include <boost/filesystem.hpp>
#include "CryConfig.h"
#include <cpp-utils/crypto/symmetric/ciphers.h>
#include <cpp-utils/either.h>
#include "cryfs/impl/config/crypto/CryConfigEncryptorFactory.h"

namespace cryfs {
    class CryConfigFile final {
    public:
        CryConfigFile(boost::filesystem::path path, CryConfig config, cpputils::unique_ref<CryConfigEncryptor> encryptor);

        CryConfigFile(CryConfigFile &&rhs) = default;
        ~CryConfigFile();

        static cpputils::unique_ref<CryConfigFile> create(boost::filesystem::path path, CryConfig config, CryKeyProvider* keyProvider);

        enum class LoadError {ConfigFileNotFound, DecryptionFailed};
        static cpputils::either<LoadError, cpputils::unique_ref<CryConfigFile>> load(boost::filesystem::path path, CryKeyProvider* keyProvider);

        void save() const;

        CryConfig *config();
        const CryConfig *config() const;

    private:
        boost::filesystem::path _path;
        CryConfig _config;
        cpputils::unique_ref<CryConfigEncryptor> _encryptor;

        DISALLOW_COPY_AND_ASSIGN(CryConfigFile);
    };
}

#endif
