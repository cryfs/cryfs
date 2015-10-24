#include "DerivedKeyConfig.h"
#include <messmer/cpp-utils/assert/assert.h>

using std::istream;
using std::ostream;
using cpputils::Data;

namespace cryfs {
    void DerivedKeyConfig::save(ostream &stream) const {
        uint8_t saltSize = _salt.size();
        ASSERT(saltSize < std::numeric_limits<uint8_t>::max(), "We don't support salts bigger than 256 byte");
        stream.write(reinterpret_cast<const char *>(&saltSize), sizeof(saltSize));
        stream.write(static_cast<const char *>(_salt.data()), saltSize);
        stream.write(reinterpret_cast<const char *>(&_N), sizeof(_N));
        stream.write(reinterpret_cast<const char *>(&_r), sizeof(_r));
        stream.write(reinterpret_cast<const char *>(&_p), sizeof(_p));
    }

    DerivedKeyConfig DerivedKeyConfig::load(istream &stream) {
        uint8_t saltSize;
        stream.read(reinterpret_cast<char *>(&saltSize), sizeof(saltSize));
        Data salt(saltSize);
        stream.read(static_cast<char *>(salt.data()), saltSize);
        decltype(_N) N;
        stream.read(reinterpret_cast<char *>(&N), sizeof(_N));
        decltype(_r) r;
        stream.read(reinterpret_cast<char *>(&r), sizeof(_r));
        decltype(_p) p;
        stream.read(reinterpret_cast<char *>(&p), sizeof(_p));
        return DerivedKeyConfig(std::move(salt), N, r, p);
    }
}
