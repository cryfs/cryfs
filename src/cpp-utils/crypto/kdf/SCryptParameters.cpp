#include "SCryptParameters.h"

using std::istream;
using std::ostream;
using cpputils::Data;

namespace cpputils {
    Data SCryptParameters::serialize() const {
        Serializer serializer(_serializedSize());
        serializer.writeUint64(_n);
        serializer.writeUint32(_r);
        serializer.writeUint32(_p);
        serializer.writeTailData(_salt);
        return serializer.finished();
    }

    size_t SCryptParameters::_serializedSize() const {
        return _salt.size() + sizeof(uint64_t) + sizeof(uint32_t) + sizeof(uint32_t);
    }

    SCryptParameters SCryptParameters::deserialize(const cpputils::Data &data) {
        Deserializer deserializer(&data);
        uint64_t n = deserializer.readUint64();
        uint32_t r = deserializer.readUint32();
        uint32_t p = deserializer.readUint32();
        Data salt = deserializer.readTailData();
        deserializer.finished();
        return SCryptParameters(std::move(salt), n, r, p);
    }

#ifndef CRYFS_NO_COMPATIBILITY
    SCryptParameters SCryptParameters::deserializeOldFormat(Deserializer *source) {
        uint64_t n = source->readUint64();
        uint32_t r = source->readUint32();
        uint32_t p = source->readUint32();
        Data salt = source->readData();
        return SCryptParameters(std::move(salt), n, r, p);
    }
#endif
}
