#pragma once
#ifndef MESSMER_CPPUTILS_CRYPTO_KDF_SCRYPT_H
#define MESSMER_CPPUTILS_CRYPTO_KDF_SCRYPT_H

#include "../../macros.h"
#include "../../random/Random.h"
#include "../../pointer/unique_ref.h"
#include "PasswordBasedKDF.h"

#include <stdexcept>
#include "SCryptParameters.h"

namespace cpputils {

    struct SCryptSettings {
        size_t SALT_LEN;
        uint64_t N;
        uint32_t r;
        uint32_t p;
    };

    class SCrypt final : public PasswordBasedKDF {
    public:
        static constexpr SCryptSettings ParanoidSettings = SCryptSettings {32, 1048576, 8, 16};
        static constexpr SCryptSettings DefaultSettings = SCryptSettings {32, 1048576, 4, 8};
        static constexpr SCryptSettings TestSettings = SCryptSettings {32, 1024, 1, 1};

        explicit SCrypt(const SCryptSettings& settingsForNewKeys);

        EncryptionKey deriveExistingKey(size_t keySize, const std::string& password, const Data& kdfParameters) override;
        KeyResult deriveNewKey(size_t keySize, const std::string& password) override;

    private:
        SCryptSettings _settingsForNewKeys;

        DISALLOW_COPY_AND_ASSIGN(SCrypt);
    };
}

#endif
