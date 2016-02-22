#pragma once
#ifndef MESSMER_CPPUTILS_CRYPTO_KDF_SCRYPT_H
#define MESSMER_CPPUTILS_CRYPTO_KDF_SCRYPT_H

#include "../../macros.h"
#include "../../random/Random.h"
#include "../../pointer/unique_ref.h"
#include "PasswordBasedKDF.h"

extern "C" {
    #include <scrypt/lib/crypto/crypto_scrypt.h>
}
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
        static constexpr SCryptSettings DefaultSettings = SCryptSettings {32, 1048576, 4, 1};
        static constexpr SCryptSettings TestSettings = SCryptSettings {32, 1024, 1, 1};

        static unique_ref<SCrypt> forNewKey(const SCryptSettings &settings);
        static unique_ref<SCrypt> forExistingKey(const Data &parameters);

        const Data &kdfParameters() const override;

        SCrypt(SCryptParameters config);

    protected:
        void derive(void *destination, size_t size, const std::string &password) override;

    private:
        void _checkCallOnlyOnce();

        SCryptParameters _config;
        Data _serializedConfig;
        bool _wasGeneratedBefore;

        DISALLOW_COPY_AND_ASSIGN(SCrypt);
    };
}

#endif
