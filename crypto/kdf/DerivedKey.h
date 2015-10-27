#pragma once
#ifndef MESSMER_CPPUTILS_CRYPTO_KDF_DERIVEDKEY_H
#define MESSMER_CPPUTILS_CRYPTO_KDF_DERIVEDKEY_H

#include "../../data/FixedSizeData.h"
#include "DerivedKeyConfig.h"

namespace cpputils {

    template<size_t KEY_LENGTH>
    class DerivedKey {
    public:
        DerivedKey(DerivedKeyConfig config, const FixedSizeData<KEY_LENGTH> &key): _config(std::move(config)), _key(key) {}
        DerivedKey(DerivedKey &&rhs) = default;

        const DerivedKeyConfig &config() const {
            return _config;
        }

        DerivedKeyConfig moveOutConfig() {
            return std::move(_config);
        }

        const FixedSizeData<KEY_LENGTH> &key() const {
            return _key;
        }

        FixedSizeData<KEY_LENGTH> moveOutKey() {
            return std::move(_key);
        }
    private:
        DerivedKeyConfig _config;
        FixedSizeData<KEY_LENGTH> _key;

        DISALLOW_COPY_AND_ASSIGN(DerivedKey);
    };
}

#endif
