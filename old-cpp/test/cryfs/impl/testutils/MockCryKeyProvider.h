#pragma once
#ifndef CRYFS_MOCKCRYKEYPROVIDER_H
#define CRYFS_MOCKCRYKEYPROVIDER_H

#include <cryfs/impl/config/CryKeyProvider.h>
#include <gmock/gmock.h>

class MockCryKeyProvider: public cryfs::CryKeyProvider {
public:
  MOCK_METHOD(cpputils::EncryptionKey, requestKeyForExistingFilesystem, (size_t keySize, const cpputils::Data& kdfParameters), (override));
  MOCK_METHOD(cryfs::CryKeyProvider::KeyResult, requestKeyForNewFilesystem, (size_t keySize), (override));
};

#endif
