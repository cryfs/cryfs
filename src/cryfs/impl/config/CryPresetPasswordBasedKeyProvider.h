#pragma once
#ifndef CRYFS_CRYPRESETPASSWORDFROMCONSOLEKEYPROVIDER_H
#define CRYFS_CRYPRESETPASSWORDFROMCONSOLEKEYPROVIDER_H

#include "CryKeyProvider.h"
#include <functional>
#include <cpp-utils/crypto/kdf/PasswordBasedKDF.h>

namespace cryfs {

    class CryPresetPasswordBasedKeyProvider final : public CryKeyProvider {
    public:
        explicit CryPresetPasswordBasedKeyProvider(std::string password, cpputils::unique_ref<cpputils::PasswordBasedKDF> kdf);

        cpputils::EncryptionKey requestKeyForExistingFilesystem(size_t keySize, const cpputils::Data& kdfParameters) override;
        KeyResult requestKeyForNewFilesystem(size_t keySize) override;

    private:
        std::string _password;
        cpputils::unique_ref<cpputils::PasswordBasedKDF> _kdf;

        DISALLOW_COPY_AND_ASSIGN(CryPresetPasswordBasedKeyProvider);
    };

}

#endif
