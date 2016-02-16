#pragma once
#ifndef MESSMER_CPPUTILS_CRYPTO_KDF_SCRYPT_H
#define MESSMER_CPPUTILS_CRYPTO_KDF_SCRYPT_H

#include "../../macros.h"
#include "../../random/Random.h"
extern "C" {
    #include <scrypt/lib/crypto/crypto_scrypt.h>
}
#include <stdexcept>
#include "DerivedKey.h"

namespace cpputils {

    struct SCryptSettings {
        size_t SALT_LEN;
        uint64_t N;
        uint32_t r;
        uint32_t p;
    };

    class SCrypt final {
    public:
        static constexpr SCryptSettings ParanoidSettings = SCryptSettings {32, 1048576, 8, 16};
        static constexpr SCryptSettings DefaultSettings = SCryptSettings {32, 1048576, 4, 1};
        static constexpr SCryptSettings TestSettings = SCryptSettings {32, 1024, 1, 1};

        SCrypt() {}

        template<size_t KEYSIZE>
        DerivedKey<KEYSIZE> generateKey(const std::string &password, const SCryptSettings &settings) {
            auto salt = Random::PseudoRandom().get(settings.SALT_LEN);
            auto config = DerivedKeyConfig(std::move(salt), settings.N, settings.r, settings.p);
            auto key = generateKeyFromConfig<KEYSIZE>(password, config);
            return DerivedKey<KEYSIZE>(std::move(config), key);
        }

        template<size_t KEYSIZE>
        FixedSizeData<KEYSIZE> generateKeyFromConfig(const std::string &password, const DerivedKeyConfig &config) {
            auto key = FixedSizeData<KEYSIZE>::Null();
            int errorcode = crypto_scrypt(reinterpret_cast<const uint8_t*>(password.c_str()), password.size(),
                          reinterpret_cast<const uint8_t*>(config.salt().data()), config.salt().size(),
                          config.N(), config.r(), config.p(),
                          static_cast<uint8_t*>(key.data()), KEYSIZE);
            if (errorcode != 0) {
                throw std::runtime_error("Error running scrypt key derivation.");
            }
            return key;
        }

    private:
        DISALLOW_COPY_AND_ASSIGN(SCrypt);
    };
}

#endif
