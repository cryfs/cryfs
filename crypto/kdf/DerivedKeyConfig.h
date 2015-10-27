#pragma once
#ifndef MESSMER_CPPUTILS_CRYPTO_KDF_KEYCONFIG_H
#define MESSMER_CPPUTILS_CRYPTO_KDF_KEYCONFIG_H

#include "../../data/Data.h"
#include "../../data/Serializer.h"
#include "../../data/Deserializer.h"
#include <iostream>

namespace cpputils {

    class DerivedKeyConfig {
    public:
        DerivedKeyConfig(Data salt, uint64_t N, uint32_t r, uint32_t p)
                : _salt(std::move(salt)),
                  _N(N), _r(r), _p(p) { }

        DerivedKeyConfig(DerivedKeyConfig &&rhs) = default;

        const Data &salt() const {
            return _salt;
        }

        size_t N() const {
            return _N;
        }

        size_t r() const {
            return _r;
        }

        size_t p() const {
            return _p;
        }

        void serialize(Serializer *destination) const;

        size_t serializedSize() const;

        static DerivedKeyConfig load(Deserializer *source);

    private:
        Data _salt;
        uint64_t _N;
        uint32_t _r;
        uint32_t _p;
    };

}

#endif
