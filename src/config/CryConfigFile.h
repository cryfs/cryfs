#pragma once
#ifndef MESSMER_CRYFS_SRC_CONFIG_CRYCONFIGFILE_H
#define MESSMER_CRYFS_SRC_CONFIG_CRYCONFIGFILE_H

#include <boost/optional.hpp>
#include <boost/filesystem.hpp>
#include "CryConfig.h"
#include <messmer/cpp-utils/crypto/symmetric/ciphers.h>
#include "crypto/CryConfigEncryptorFactory.h"

namespace cryfs {
    class CryConfigFile final {
    public:
        CryConfigFile(CryConfigFile &&rhs) = default;
        ~CryConfigFile();

        template<class SCryptConfig>
        static CryConfigFile create(const boost::filesystem::path &path, CryConfig config, const std::string &password);
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

    template<class SCryptSettings>
    CryConfigFile CryConfigFile::create(const boost::filesystem::path &path, CryConfig config, const std::string &password) {
        using ConfigCipher = cpputils::AES256_GCM; // TODO Take cipher from config instead
        if (boost::filesystem::exists(path)) {
            throw std::runtime_error("Config file exists already.");
        }
        auto result = CryConfigFile(path, std::move(config), CryConfigEncryptorFactory::deriveKey<ConfigCipher, SCryptSettings>(password));
        result.save();
        return result;
    }
}

#endif
