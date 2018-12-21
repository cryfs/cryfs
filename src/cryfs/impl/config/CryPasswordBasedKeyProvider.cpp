#include "CryPasswordBasedKeyProvider.h"

using std::shared_ptr;
using cpputils::Console;
using cpputils::unique_ref;
using cpputils::EncryptionKey;
using cpputils::unique_ref;
using cpputils::PasswordBasedKDF;

namespace cryfs {

CryPasswordBasedKeyProvider::CryPasswordBasedKeyProvider(shared_ptr<Console> console, std::function<std::string()> askPasswordForExistingFilesystem, std::function<std::string()> askPasswordForNewFilesystem, unique_ref<PasswordBasedKDF> kdf)
    : _console(std::move(console)), _askPasswordForExistingFilesystem(std::move(askPasswordForExistingFilesystem)), _askPasswordForNewFilesystem(std::move(askPasswordForNewFilesystem)), _kdf(std::move(kdf)) {}

EncryptionKey CryPasswordBasedKeyProvider::requestKeyForExistingFilesystem(size_t keySize, const cpputils::Data& kdfParameters) {
  auto password = _askPasswordForExistingFilesystem();
  _console->print("Deriving encryption key (this can take some time)...");
  auto key = _kdf->deriveExistingKey(keySize, password, kdfParameters);
  _console->print("done\n");
  return key;
}

CryKeyProvider::KeyResult CryPasswordBasedKeyProvider::requestKeyForNewFilesystem(size_t keySize) {
  auto password = _askPasswordForNewFilesystem();
  _console->print("Deriving encryption key (this can take some time)...");
  auto keyResult = _kdf->deriveNewKey(keySize, password);
  _console->print("done\n");
  return {std::move(keyResult.key), std::move(keyResult.kdfParameters)};
}

}
