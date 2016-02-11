#pragma once
#ifndef MESSMER_CPPUTILS_DATA_DESERIALIZER_H
#define MESSMER_CPPUTILS_DATA_DESERIALIZER_H

#include "Data.h"
#include "../macros.h"
#include "../assert/assert.h"

namespace cpputils {
    class Deserializer final {
    public:
        Deserializer(const Data *source);

        uint8_t readUint8();
        int8_t readInt8();
        uint16_t readUint16();
        int16_t readInt16();
        uint32_t readUint32();
        int32_t readInt32();
        uint64_t readUint64();
        int64_t readInt64();
        std::string readString();
        Data readData();
        Data readTailData();

        void finished();

    private:
        template<typename DataType> DataType _read();
        Data _readData(size_t size);

        size_t _pos;
        const Data *_source;

        DISALLOW_COPY_AND_ASSIGN(Deserializer);
    };

    inline Deserializer::Deserializer(const Data *source): _pos(0), _source(source) {
    }

    inline uint8_t Deserializer::readUint8() {
        return _read<uint8_t>();
    }

    inline int8_t Deserializer::readInt8() {
        return _read<int8_t>();
    }

    inline uint16_t Deserializer::readUint16() {
        return _read<uint16_t>();
    }

    inline int16_t Deserializer::readInt16() {
        return _read<int16_t>();
    }

    inline uint32_t Deserializer::readUint32() {
        return _read<uint32_t>();
    }

    inline int32_t Deserializer::readInt32() {
        return _read<int32_t>();
    }

    inline uint64_t Deserializer::readUint64() {
        return _read<uint64_t>();
    }

    inline int64_t Deserializer::readInt64() {
        return _read<int64_t>();
    }

    template<typename DataType>
    inline DataType Deserializer::_read() {
        static_assert(std::is_pod<DataType>::value, "Can only deserialize PODs");
        if (_pos + sizeof(DataType) > _source->size()) {
            throw std::runtime_error("Deserialization failed - size overflow");
        }
        DataType result = *reinterpret_cast<const DataType*>(_source->dataOffset(_pos));
        _pos += sizeof(DataType);
        return result;
    }

    inline Data Deserializer::readData() {
        uint64_t size = readUint64();
        if (_pos + size > _source->size()) {
            throw std::runtime_error("Deserialization failed - size overflow");
        }
        return _readData(size);
    }

    inline Data Deserializer::readTailData() {
        uint64_t size = _source->size() - _pos;
        return _readData(size);
    }

    inline Data Deserializer::_readData(size_t size) {
        Data result(size);
        std::memcpy(static_cast<char*>(result.data()), static_cast<const char*>(_source->dataOffset(_pos)), size);
        _pos += size;
        return result;
    }

    inline std::string Deserializer::readString() {
        //TODO Test whether that works when string ends (a) at beginning (b) in middle (c) at end of data region
        const void *nullbytepos = std::memchr(_source->dataOffset(_pos), '\0', _source->size()-_pos);
        if (nullbytepos == nullptr) {
            throw std::runtime_error("Deserialization failed - missing nullbyte for string termination");
        }
        uint64_t size = static_cast<const uint8_t*>(nullbytepos) - static_cast<const uint8_t*>(_source->dataOffset(_pos));
        std::string result(reinterpret_cast<const char*>(_source->dataOffset(_pos)), size);
        _pos += size + 1;
        return result;
    }

    inline void Deserializer::finished() {
        if (_pos != _source->size()) {
            throw std::runtime_error("Deserialization failed - size not fully used.");
        }
    }
}

#endif
