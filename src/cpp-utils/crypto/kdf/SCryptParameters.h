#pragma once
#ifndef MESSMER_CPPUTILS_CRYPTO_KDF_KEYCONFIG_H
#define MESSMER_CPPUTILS_CRYPTO_KDF_KEYCONFIG_H

#include "../../data/Data.h"
#include "../../data/Serializer.h"
#include "../../data/Deserializer.h"
#include <iostream>

namespace cpputils {

    //TODO Test Copy/move constructor and assignment
    //TODO Test operator==/!=
    //TODO Use SCryptSettings as a member here instead of storing _N, _r, _p.

    class SCryptParameters final {
    public:
        SCryptParameters(Data salt, uint64_t N, uint32_t r, uint32_t p)
                : _salt(std::move(salt)),
                  _N(N), _r(r), _p(p) { }

        SCryptParameters(const SCryptParameters &rhs)
                :_salt(rhs._salt.copy()),
                 _N(rhs._N), _r(rhs._r), _p(rhs._p) { }

        SCryptParameters(SCryptParameters &&rhs) = default;

        SCryptParameters &operator=(const SCryptParameters &rhs) {
            _salt = rhs._salt.copy();
            _N = rhs._N;
            _r = rhs._r;
            _p = rhs._p;
            return *this;
        }

        SCryptParameters &operator=(SCryptParameters &&rhs) = default;

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

        cpputils::Data serialize() const;
        static SCryptParameters deserialize(const cpputils::Data &data);

#ifndef CRYFS_NO_COMPATIBILITY
        static SCryptParameters deserializeOldFormat(cpputils::Deserializer *deserializer);
        size_t serializedSize() const {
            return _serializedSize();
        }
#endif

    private:
        size_t _serializedSize() const;

        Data _salt;
        uint64_t _N;
        uint32_t _r;
        uint32_t _p;
    };

    inline bool operator==(const SCryptParameters &lhs, const SCryptParameters &rhs) {
        return lhs.salt() == rhs.salt() && lhs.N() == rhs.N() && lhs.r() == rhs.r() && lhs.p() == rhs.p();
    }

    inline bool operator!=(const SCryptParameters &lhs, const SCryptParameters &rhs) {
        return !operator==(lhs, rhs);
    }

}

#endif
