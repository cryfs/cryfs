#pragma once
#ifndef MESSMER_CRYFS_SRC_CONFIG_CRYCONFIGFILE_H
#define MESSMER_CRYFS_SRC_CONFIG_CRYCONFIGFILE_H

#include <boost/optional.hpp>
#include <boost/filesystem.hpp>
#include "CryConfig.h"
#include <cpp-utils/crypto/symmetric/ciphers.h>
#include "crypto/CryConfigEncryptorFactory.h"
#include <cpp-utils/either.h>

namespace cryfs {
    class CryConfigFile final {
    public:
        CryConfigFile(const boost::filesystem::path &path, const CryConfig &config, cpputils::unique_ref<CryConfigEncryptor> encryptor);

        CryConfigFile(CryConfigFile &&rhs) = default;
        ~CryConfigFile();

        static cpputils::unique_ref<CryConfigFile> create(const boost::filesystem::path &path, const CryConfig &config, const std::string &password, const cpputils::SCryptSettings &scryptSettings);
        enum class LoadError {ConfigFileNotFound, DecryptionFailed};
        static cpputils::either<LoadError, cpputils::unique_ref<CryConfigFile>> load(const boost::filesystem::path &path, const std::string &password);
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
