#pragma once
#ifndef CRYFS_TEST_LIBUSAGETEST_TESTUTILS_FILESYSTEMHELPER_H
#define CRYFS_TEST_LIBUSAGETEST_TESTUTILS_FILESYSTEMHELPER_H

#include <blockstore/implementations/ondisk/OnDiskBlockStore2.h>
#include <cryfs/impl/filesystem/CryDevice.h>
#include <cryfs/impl/config/CryConfig.h>
#include <cryfs/impl/config/CryConfigFile.h>

const std::string PASSWORD = "mypassword";

inline std::shared_ptr<cryfs::CryConfigFile> create_configfile(const boost::filesystem::path &configfile_path, const string &cipher = "aes-256-gcm") {
    cryfs::CryConfig config;
    config.SetCipher(cipher);
    config.SetEncryptionKey(cryfs::CryCiphers::find(cipher).createKey(cpputils::Random::PseudoRandom()));
    config.SetRootBlob("");
    config.SetBlocksizeBytes(32*1024);
    config.SetVersion(cryfs::CryConfig::FilesystemFormatVersion);
    config.SetCreatedWithVersion(gitversion::VersionString());
    config.SetLastOpenedWithVersion(gitversion::VersionString());
    return cryfs::CryConfigFile::create(configfile_path, std::move(config), PASSWORD, cpputils::SCrypt::TestSettings);
}

inline std::shared_ptr<cryfs::CryConfigFile> create_configfile_for_incompatible_cryfs_version(const boost::filesystem::path &configfile_path) {
    cryfs::CryConfig config;
    config.SetCipher("aes-256-gcm");
    config.SetEncryptionKey(cpputils::AES256_GCM::EncryptionKey::CreateKey(cpputils::Random::PseudoRandom()).ToString());
    config.SetRootBlob("");
    config.SetVersion("0.8.0");
    config.SetCreatedWithVersion("0.8.0");
    config.SetLastOpenedWithVersion("0.8.0");
    return cryfs::CryConfigFile::create(configfile_path, std::move(config), PASSWORD, cpputils::SCrypt::TestSettings);
}

inline void create_filesystem(const boost::filesystem::path &basedir, const boost::optional<boost::filesystem::path> &configfile_path = boost::none, const std::string &cipher = "aes-256-gcm") {
    boost::filesystem::path actual_configfile_path;
    if (configfile_path == boost::none) {
        actual_configfile_path = basedir / "cryfs.config";
    } else {
        actual_configfile_path = *configfile_path;
    }
    auto configfile = create_configfile(actual_configfile_path, cipher);
    auto blockstore = cpputils::make_unique_ref<blockstore::ondisk::OnDiskBlockStore2>(basedir);

    // TODO Do the things in the next block need to be configurable? What does the test expect?
    // The test was unfortunately written before these options were added to CryFS.
    cpputils::TempDir tempDir;
    cryfs::LocalStateDir localStateDir(tempDir.path());
    uint32_t myClientId = 0x12345678;
    bool allowIntegrityViolation = false;
    bool missingBlockIsIntegrityViolation = false;

    cryfs::CryDevice device(std::move(configfile), std::move(blockstore), std::move(localStateDir), myClientId, allowIntegrityViolation, missingBlockIsIntegrityViolation);
}

#endif
