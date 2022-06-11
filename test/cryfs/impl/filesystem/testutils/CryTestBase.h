#ifndef MESSMER_CRYFS_TEST_CRYFS_FILESYSTEM_CRYTESTBASE_H
#define MESSMER_CRYFS_TEST_CRYFS_FILESYSTEM_CRYTESTBASE_H

#include <cryfs/impl/filesystem/CryDevice.h>
#include <cryfs/impl/filesystem/CryDir.h>
#include <cryfs/impl/filesystem/CryNode.h>
#include <cryfs/impl/filesystem/CryOpenFile.h>
#include <cryfs/impl/config/CryPresetPasswordBasedKeyProvider.h>
#include <blockstore/implementations/inmemory/InMemoryBlockStore2.h>
#include <cpp-utils/tempfile/TempFile.h>
#include <cpp-utils/crypto/kdf/Scrypt.h>
#include "../../testutils/TestWithFakeHomeDirectory.h"
#include "../../testutils/MockConsole.h"

inline auto failOnIntegrityViolation() {
  return [] {
    EXPECT_TRUE(false);
  };
}

class CryTestBase : public TestWithFakeHomeDirectory {
public:
    CryTestBase(): _tempLocalStateDir(), _localStateDir(_tempLocalStateDir.path()), _configFile(false), _device(nullptr) {
        _device = std::make_unique<cryfs::CryDevice>(configFile(), _localStateDir, 0x12345678, false, false, failOnIntegrityViolation());
        _device->setContext(fspp::Context { fspp::relatime() });
    }

    std::shared_ptr<cryfs::CryConfigFile> configFile() {
        cryfs::CryConfig config;
        config.SetCipher("aes-256-gcm");
        config.SetEncryptionKey(cpputils::AES256_GCM::EncryptionKey::CreateKey(cpputils::Random::PseudoRandom(), cpputils::AES256_GCM::KEYSIZE).ToString());
        config.SetBlocksizeBytes(10240);
        cryfs::CryPresetPasswordBasedKeyProvider keyProvider("mypassword", cpputils::make_unique_ref<cpputils::SCrypt>(cpputils::SCrypt::TestSettings));
        return cryfs::CryConfigFile::create(_configFile.path(), std::move(config), &keyProvider);
    }

    cryfs::CryDevice &device() {
        return *_device;
    }

    static constexpr fspp::mode_t MODE_PUBLIC = fspp::mode_t()
            .addUserReadFlag().addUserWriteFlag().addUserExecFlag()
            .addGroupReadFlag().addGroupWriteFlag().addGroupExecFlag()
            .addOtherReadFlag().addOtherWriteFlag().addOtherExecFlag();

    cpputils::unique_ref<cryfs::CryNode> CreateFile(const boost::filesystem::path &path) {
        auto parentDir = device().LoadDir(path.parent_path()).value();
        parentDir->createAndOpenFile(path.filename().string(), MODE_PUBLIC, fspp::uid_t(0), fspp::gid_t(0));
        auto file = device().Load(path).value();
        return cpputils::dynamic_pointer_move<cryfs::CryNode>(file).value();
    }

    cpputils::unique_ref<cryfs::CryNode> CreateDir(const boost::filesystem::path &path) {
        auto _parentDir = device().Load(path.parent_path()).value();
        auto parentDir = cpputils::dynamic_pointer_move<cryfs::CryDir>(_parentDir).value();
        parentDir->createDir(path.filename().string(), MODE_PUBLIC, fspp::uid_t(0), fspp::gid_t(0));
        auto createdDir = device().Load(path).value();
        return cpputils::dynamic_pointer_move<cryfs::CryNode>(createdDir).value();
    }

    cpputils::unique_ref<cryfs::CryNode> CreateSymlink(const boost::filesystem::path &path) {
        auto _parentDir = device().Load(path.parent_path()).value();
        auto parentDir = cpputils::dynamic_pointer_move<cryfs::CryDir>(_parentDir).value();
        parentDir->createSymlink(path.filename().string(), "/target", fspp::uid_t(0), fspp::gid_t(0));
        auto createdSymlink = device().Load(path).value();
        return cpputils::dynamic_pointer_move<cryfs::CryNode>(createdSymlink).value();
    }

    bool Exists(const boost::filesystem::path &path) {
        return device().Load(path) != boost::none;
    }

private:
    cpputils::TempDir _tempLocalStateDir;
    cryfs::LocalStateDir _localStateDir;
    cpputils::TempFile _configFile;
    std::unique_ptr<cryfs::CryDevice> _device;
};

#endif
