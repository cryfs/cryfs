#pragma once
#ifndef CRYFS_FAKECRYKEYPROVIDER_H
#define CRYFS_FAKECRYKEYPROVIDER_H

#include <cryfs/config/CryKeyProvider.h>
#include <cpp-utils/data/DataFixture.h>

class FakeCryKeyProvider final : public cryfs::CryKeyProvider {
private:
  static constexpr const unsigned char KDF_TEST_PARAMETERS = 5; // test value to check that kdf parameters are passed in correctly
public:
  FakeCryKeyProvider(unsigned char keySeed = 0): _keySeed(keySeed) {}

  cpputils::EncryptionKey requestKeyForExistingFilesystem(size_t keySize, const cpputils::Data& kdfParameters) override {
    ASSERT(kdfParameters.size() == 1 && *reinterpret_cast<const unsigned char*>(kdfParameters.data()) == KDF_TEST_PARAMETERS, "Wrong kdf parameters");

    return cpputils::EncryptionKey::FromString(cpputils::DataFixture::generate(keySize, _keySeed).ToString());
  }

  KeyResult requestKeyForNewFilesystem(size_t keySize) override {
    cpputils::Data kdfParameters(sizeof(unsigned char));
    *reinterpret_cast<unsigned char*>(kdfParameters.data()) = KDF_TEST_PARAMETERS;

    auto key = requestKeyForExistingFilesystem(keySize, kdfParameters);
    return KeyResult{
        std::move(key),
        std::move(kdfParameters)
    };
  }

private:
  unsigned char _keySeed;
};

#endif
