#pragma once
#ifndef CRYFS_CRYPASSWORDFROMCONSOLEKEYPROVIDER_H
#define CRYFS_CRYPASSWORDFROMCONSOLEKEYPROVIDER_H

#include "CryKeyProvider.h"
#include <functional>
#include <cpp-utils/crypto/kdf/PasswordBasedKDF.h>
#include <cpp-utils/io/Console.h>

namespace cryfs {

// TODO Remove duplication with CryPresetPasswordBasedKeyProvider
class CryPasswordBasedKeyProvider final : public CryKeyProvider {
public:
  explicit CryPasswordBasedKeyProvider(std::shared_ptr<cpputils::Console> console, std::function<std::string()> askPasswordForExistingFilesystem, std::function<std::string()> askPasswordForNewFilesystem, cpputils::unique_ref<cpputils::PasswordBasedKDF> kdf);

  cpputils::EncryptionKey requestKeyForExistingFilesystem(size_t keySize, const cpputils::Data& kdfParameters) override;
  KeyResult requestKeyForNewFilesystem(size_t keySize) override;

private:
  std::shared_ptr<cpputils::Console> _console;
  std::function<std::string()> _askPasswordForExistingFilesystem;
  std::function<std::string()> _askPasswordForNewFilesystem;
  cpputils::unique_ref<cpputils::PasswordBasedKDF> _kdf;

  DISALLOW_COPY_AND_ASSIGN(CryPasswordBasedKeyProvider);
};

}

#endif
