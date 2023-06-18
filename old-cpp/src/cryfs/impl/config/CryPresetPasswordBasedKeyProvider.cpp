#include "CryPresetPasswordBasedKeyProvider.h"

using cpputils::unique_ref;
using cpputils::EncryptionKey;
using cpputils::unique_ref;
using cpputils::PasswordBasedKDF;
using cpputils::Data;

namespace cryfs {

CryPresetPasswordBasedKeyProvider::CryPresetPasswordBasedKeyProvider(std::string password, unique_ref<PasswordBasedKDF> kdf)
: _password(std::move(password)), _kdf(std::move(kdf)) {}

EncryptionKey CryPresetPasswordBasedKeyProvider::requestKeyForExistingFilesystem(size_t keySize, const Data& kdfParameters) {
    return _kdf->deriveExistingKey(keySize, _password, kdfParameters);
}

CryPresetPasswordBasedKeyProvider::KeyResult CryPresetPasswordBasedKeyProvider::requestKeyForNewFilesystem(size_t keySize) {
    auto keyResult = _kdf->deriveNewKey(keySize, _password);
    return {std::move(keyResult.key), std::move(keyResult.kdfParameters)};
}

}
