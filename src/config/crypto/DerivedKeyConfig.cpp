#include "DerivedKeyConfig.h"
#include <messmer/cpp-utils/assert/assert.h>
#include <messmer/cpp-utils/logging/logging.h>

using std::istream;
using std::ostream;
using cpputils::Data;
using namespace cpputils::logging;

namespace cryfs {
    Data DerivedKeyConfig::save() const {
        uint8_t saltSize = _salt.size();
        ASSERT(saltSize < std::numeric_limits<uint8_t>::max(), "We don't support salts bigger than 256 byte");
        Data result(sizeof(saltSize) + saltSize + sizeof(_N) + sizeof(_r) + sizeof(_p));
        //TODO Write and use a DataSerializer class with DataSerializer::write<uint8_t> and so on instead of all these memcpy(reinterpret_cast).
        std::memcpy(reinterpret_cast<char*>(result.data()), reinterpret_cast<const char*>(&saltSize), sizeof(saltSize));
        std::memcpy(reinterpret_cast<char*>(result.dataOffset(sizeof(saltSize))), reinterpret_cast<const char*>(_salt.data()), saltSize);
        std::memcpy(reinterpret_cast<char*>(result.dataOffset(sizeof(saltSize)+saltSize)), reinterpret_cast<const char*>(&_N), sizeof(_N));
        std::memcpy(reinterpret_cast<char*>(result.dataOffset(sizeof(saltSize)+saltSize+sizeof(_N))), reinterpret_cast<const char*>(&_r), sizeof(_r));
        std::memcpy(reinterpret_cast<char*>(result.dataOffset(sizeof(saltSize)+saltSize+sizeof(_N)+sizeof(_r))), reinterpret_cast<const char*>(&_p), sizeof(_p));
        return result;
    }

    boost::optional<DerivedKeyConfig> DerivedKeyConfig::load(const Data &data) {
        uint8_t saltSize;
        //TODO Write and use a DataDeserializer class instead of all these memcpy(reinterpret_cast).
        std::memcpy(reinterpret_cast<char*>(&saltSize), reinterpret_cast<const char*>(data.data()), sizeof(saltSize));
        Data salt(saltSize);
        if (sizeof(saltSize) + saltSize + sizeof(_N) + sizeof(_p) + sizeof(_r) != data.size()) {
            LOG(ERROR) << "Could not load DerivedKeyConfig. Wrong size.";
            return boost::none;
        }
        std::memcpy(reinterpret_cast<char*>(salt.data()), reinterpret_cast<const char*>(data.dataOffset(sizeof(saltSize))), saltSize);
        decltype(_N) N;
        std::memcpy(reinterpret_cast<char*>(&N), reinterpret_cast<const char*>(data.dataOffset(sizeof(saltSize)+saltSize)), sizeof(_N));
        decltype(_r) r;
        std::memcpy(reinterpret_cast<char*>(&r), reinterpret_cast<const char*>(data.dataOffset(sizeof(saltSize)+saltSize+sizeof(_N))), sizeof(_r));
        decltype(_p) p;
        std::memcpy(reinterpret_cast<char*>(&p), reinterpret_cast<const char*>(data.dataOffset(sizeof(saltSize)+saltSize+sizeof(_N)+sizeof(_r))), sizeof(_p));
        return DerivedKeyConfig(std::move(salt), N, r, p);
    }
}
