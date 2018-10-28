#pragma once
#ifndef CRYFS_CRYPRESETPASSWORDFROMCONSOLEKEYPROVIDER_H
#define CRYFS_CRYPRESETPASSWORDFROMCONSOLEKEYPROVIDER_H

#include "CryKeyProvider.h"
#include <functional>
#include <cpp-utils/crypto/kdf/Scrypt.h>
#include <cpp-utils/io/Console.h>

namespace cryfs {

class CryPresetPasswordBasedKeyProvider final : public CryKeyProvider {
public:
    explicit CryPresetPasswordBasedKeyProvider(std::string password, cpputils::unique_ref<cpputils::PasswordBasedKDF> kdf)
            : _password(std::move(password)), _kdf(std::move(kdf)) {}

    cpputils::EncryptionKey requestKeyForExistingFilesystem(size_t keySize, const cpputils::Data& kdfParameters) override {
        return _kdf->deriveExistingKey(keySize, _password, kdfParameters);
    }

    KeyResult requestKeyForNewFilesystem(size_t keySize) override {
        auto keyResult = _kdf->deriveNewKey(keySize, _password);
        return {std::move(keyResult.key), std::move(keyResult.kdfParameters)};
    }

private:
    std::string _password;
    cpputils::unique_ref<cpputils::PasswordBasedKDF> _kdf;

    DISALLOW_COPY_AND_ASSIGN(CryPresetPasswordBasedKeyProvider);
};

}

#endif
