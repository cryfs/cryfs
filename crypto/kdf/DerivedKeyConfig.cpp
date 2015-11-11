#include "DerivedKeyConfig.h"

using std::istream;
using std::ostream;
using boost::optional;
using boost::none;

namespace cpputils {
    void DerivedKeyConfig::serialize(Serializer *target) const {
        target->writeUint64(_N);
        target->writeUint32(_r);
        target->writeUint32(_p);
        target->writeData(_salt);
    }

    size_t DerivedKeyConfig::serializedSize() const {
        return Serializer::DataSize(_salt) + sizeof(uint64_t) + sizeof(uint32_t) + sizeof(uint32_t);
    }

    DerivedKeyConfig DerivedKeyConfig::deserialize(Deserializer *source) {
        uint64_t N = source->readUint64();
        uint32_t r = source->readUint32();
        uint32_t p = source->readUint32();
        Data salt = source->readData();
        return DerivedKeyConfig(std::move(salt), N, r, p);
    }
}
