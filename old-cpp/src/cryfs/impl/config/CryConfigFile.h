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
        enum class Access : uint8_t {
            // Never write to the config file, just read it.
            // Note that this is only sound if the file system itself
            // is also loaded read-only, or at least with migrations disabled.
            // Otherwise, the file system might get migrated but the config
            // file will still say it's the old version.
            ReadOnly,

            // Load the config file and update it if necessary,
            // e.g. write the "last opened with" entry into it
            // and potentially upgrade the version number.
            ReadWrite,
        };

        CryConfigFile(boost::filesystem::path path, CryConfig config, cpputils::unique_ref<CryConfigEncryptor> encryptor, Access access);

        CryConfigFile(CryConfigFile &&rhs) = default;
        ~CryConfigFile();

        static cpputils::unique_ref<CryConfigFile> create(boost::filesystem::path path, CryConfig config, CryKeyProvider* keyProvider);

        enum class LoadError {ConfigFileNotFound, DecryptionFailed};
        static cpputils::either<LoadError, cpputils::unique_ref<CryConfigFile>> load(boost::filesystem::path path, CryKeyProvider* keyProvider, Access access);

        void save() const;

        CryConfig *config();
        const CryConfig *config() const;

    private:
        boost::filesystem::path _path;
        CryConfig _config;
        cpputils::unique_ref<CryConfigEncryptor> _encryptor;
        Access _access;

        DISALLOW_COPY_AND_ASSIGN(CryConfigFile);
    };
}

#endif
