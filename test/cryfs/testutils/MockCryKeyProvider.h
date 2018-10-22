#pragma once
#ifndef CRYFS_MOCKCRYKEYPROVIDER_H
#define CRYFS_MOCKCRYKEYPROVIDER_H

#include <cryfs/config/CryKeyProvider.h>
#include <gmock/gmock.h>

class MockCryKeyProvider: public cryfs::CryKeyProvider {
public:
  MOCK_METHOD2(requestKeyForExistingFilesystem, cpputils::EncryptionKey(size_t keySize, const cpputils::Data& kdfParameters));
  MOCK_METHOD1(requestKeyForNewFilesystem, cryfs::CryKeyProvider::KeyResult(size_t keySize));
};

#endif
