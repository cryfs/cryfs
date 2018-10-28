#ifndef MESSMER_CRYFS_TEST_CRYFS_FILESYSTEM_CRYTESTBASE_H
#define MESSMER_CRYFS_TEST_CRYFS_FILESYSTEM_CRYTESTBASE_H

#include <cryfs/filesystem/CryDevice.h>
#include <cryfs/config/CryPresetPasswordBasedKeyProvider.h>
#include <blockstore/implementations/inmemory/InMemoryBlockStore2.h>
#include <cpp-utils/tempfile/TempFile.h>
#include <cpp-utils/crypto/kdf/Scrypt.h>
#include "../../testutils/TestWithFakeHomeDirectory.h"
#include "../../testutils/MockConsole.h"

class CryTestBase : public TestWithFakeHomeDirectory {
public:
    CryTestBase(): _tempLocalStateDir(), _localStateDir(_tempLocalStateDir.path()), _configFile(false), _device(nullptr) {
        auto fakeBlockStore = cpputils::make_unique_ref<blockstore::inmemory::InMemoryBlockStore2>();
        _device = std::make_unique<cryfs::CryDevice>(configFile(), std::move(fakeBlockStore), _localStateDir, 0x12345678, false, false);
    }

    cryfs::CryConfigFile configFile() {
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

private:
    cpputils::TempDir _tempLocalStateDir;
    cryfs::LocalStateDir _localStateDir;
    cpputils::TempFile _configFile;
    std::unique_ptr<cryfs::CryDevice> _device;
};

#endif
