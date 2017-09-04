#ifndef MESSMER_CRYFS_TEST_CRYFS_FILESYSTEM_CRYTESTBASE_H
#define MESSMER_CRYFS_TEST_CRYFS_FILESYSTEM_CRYTESTBASE_H

#include <cryfs/filesystem/CryDevice.h>
#include <blockstore/implementations/inmemory/InMemoryBlockStore2.h>
#include <cpp-utils/tempfile/TempFile.h>
#include <cpp-utils/crypto/kdf/Scrypt.h>
#include "../../testutils/TestWithFakeHomeDirectory.h"

class CryTestBase : public TestWithFakeHomeDirectory {
public:
    CryTestBase(): _configFile(false), _device(nullptr) {
        auto fakeBlockStore = cpputils::make_unique_ref<blockstore::inmemory::InMemoryBlockStore2>();
        _device = std::make_unique<cryfs::CryDevice>(configFile(), std::move(fakeBlockStore), 0x12345678);
    }

    cryfs::CryConfigFile configFile() {
        cryfs::CryConfig config;
        config.SetCipher("aes-256-gcm");
        config.SetEncryptionKey(cpputils::AES256_GCM::EncryptionKey::CreateKey(cpputils::Random::PseudoRandom()).ToString());
        config.SetBlocksizeBytes(10240);
        return cryfs::CryConfigFile::create(_configFile.path(), std::move(config), "mypassword", cpputils::SCrypt::TestSettings);
    }

    cryfs::CryDevice &device() {
        return *_device;
    }

private:
    cpputils::TempFile _configFile;
    std::unique_ptr<cryfs::CryDevice> _device;
};

#endif
