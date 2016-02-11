#pragma once
#ifndef MESSMER_CRYFS_SRC_CONFIG_CRYCONFIGFILE_H
#define MESSMER_CRYFS_SRC_CONFIG_CRYCONFIGFILE_H

#include <boost/optional.hpp>
#include <boost/filesystem.hpp>
#include "CryConfig.h"
#include <cpp-utils/crypto/symmetric/ciphers.h>
#include "crypto/CryConfigEncryptorFactory.h"

namespace cryfs {
    class CryConfigFile final {
    public:
        CryConfigFile(CryConfigFile &&rhs) = default;
        ~CryConfigFile();

        static CryConfigFile create(const boost::filesystem::path &path, CryConfig config, const std::string &password, const cpputils::SCryptSettings &scryptSettings);
        static boost::optional<CryConfigFile> load(const boost::filesystem::path &path, const std::string &password);
        void save() const;

        CryConfig *config();

    private:
        CryConfigFile(const boost::filesystem::path &path, CryConfig config, cpputils::unique_ref<CryConfigEncryptor> encryptor);

        boost::filesystem::path _path;
        CryConfig _config;
        cpputils::unique_ref<CryConfigEncryptor> _encryptor;

        DISALLOW_COPY_AND_ASSIGN(CryConfigFile);
    };
}

#endif
