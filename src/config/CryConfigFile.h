#pragma once
#ifndef MESSMER_CRYFS_SRC_CONFIG_CRYCONFIGFILE_H
#define MESSMER_CRYFS_SRC_CONFIG_CRYCONFIGFILE_H

#include <boost/optional.hpp>
#include <boost/filesystem/path.hpp>
#include "CryConfig.h"
#include "CryConfigEncryptor.h"
#include <messmer/blockstore/implementations/encrypted/ciphers/ciphers.h>

namespace cryfs {
    class CryConfigFile final {
    public:
        using ConfigCipher = blockstore::encrypted::AES256_GCM;
        using ConfigEncryptionKey = DerivedKey<ConfigCipher::EncryptionKey::BINARY_LENGTH>;

        CryConfigFile(CryConfigFile &&rhs) = default;
        ~CryConfigFile();

        static CryConfigFile create(const boost::filesystem::path &path, CryConfig config, const std::string &password);
        static boost::optional<CryConfigFile> load(const boost::filesystem::path &path, const std::string &password);
        void save() const;

        CryConfig *config();

    private:
        CryConfigFile(const boost::filesystem::path &path, CryConfig config, ConfigEncryptionKey configEncKey);

        static ConfigEncryptionKey _deriveKey(const std::string &password);
        static std::string _decrypt(cpputils::Data content, const ConfigEncryptionKey &configEncKey);
        static cpputils::Data _encrypt(const std::string &content, const ConfigEncryptionKey &configEncKey);

        boost::filesystem::path _path;
        CryConfig _config;
        ConfigEncryptionKey _configEncKey;
        using Encryptor = CryConfigEncryptor<ConfigCipher>;

        DISALLOW_COPY_AND_ASSIGN(CryConfigFile);
    };
}

#endif
