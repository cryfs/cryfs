#pragma once
#ifndef MESSMER_CPPUTILS_DATA_SERIALIZER_H
#define MESSMER_CPPUTILS_DATA_SERIALIZER_H

#include "Data.h"
#include "../macros.h"
#include "../assert/assert.h"
#include <string>

namespace cpputils {
    //TODO Test Serializer/Deserializer
    //TODO Also test system (big endian/little endian) by adding a serialized data file to the repository and (a) reading it and (b) rewriting and comparing it
    class Serializer final {
    public:
        Serializer(size_t size);

        void writeUint8(uint8_t value);
        void writeInt8(int8_t value);
        void writeUint16(uint16_t value);
        void writeInt16(int16_t value);
        void writeUint32(uint32_t value);
        void writeInt32(int32_t value);
        void writeUint64(uint64_t value);
        void writeInt64(int64_t value);
        void writeString(const std::string &value);
        void writeData(const Data &value);

        // Write the data as last element when serializing.
        // It does not store a data size but limits the size by the size of the serialization result
        void writeTailData(const Data &value);

        static size_t DataSize(const Data &value);
        static size_t StringSize(const std::string &value);

        Data finished();

    private:
        template<typename DataType> void _write(DataType obj);
        void _writeData(const Data &value);

        size_t _pos;
        Data _result;

        DISALLOW_COPY_AND_ASSIGN(Serializer);
    };

    inline Serializer::Serializer(size_t size): _pos(0), _result(size) {
    }

    inline void Serializer::writeUint8(uint8_t value) {
        _write<uint8_t>(value);
    }

    inline void Serializer::writeInt8(int8_t value) {
        _write<int8_t>(value);
    }

    inline void Serializer::writeUint16(uint16_t value) {
        _write<uint16_t>(value);
    }

    inline void Serializer::writeInt16(int16_t value) {
        _write<int16_t>(value);
    }

    inline void Serializer::writeUint32(uint32_t value) {
        _write<uint32_t>(value);
    }

    inline void Serializer::writeInt32(int32_t value) {
        _write<int32_t>(value);
    }

    inline void Serializer::writeUint64(uint64_t value) {
        _write<uint64_t>(value);
    }

    inline void Serializer::writeInt64(int64_t value) {
        _write<int64_t>(value);
    }

    template<typename DataType>
    inline void Serializer::_write(DataType obj) {
        static_assert(std::is_pod<DataType>::value, "Can only serialize PODs");
        if (_pos + sizeof(DataType) > _result.size()) {
            throw std::runtime_error("Serialization failed - size overflow");
        }
        *reinterpret_cast<DataType*>(_result.dataOffset(_pos)) = obj;
        _pos += sizeof(DataType);
    }

    inline void Serializer::writeData(const Data &data) {
        writeUint64(data.size());
        _writeData(data);
    }

    inline size_t Serializer::DataSize(const Data &data) {
        return sizeof(uint64_t) + data.size();
    }

    inline void Serializer::writeTailData(const Data &data) {
        ASSERT(_pos + data.size() == _result.size(), "Not enough data given to write until the end of the stream");
        _writeData(data);
    }

    inline void Serializer::_writeData(const Data &data) {
        if (_pos + data.size() > _result.size()) {
            throw std::runtime_error("Serialization failed - size overflow");
        }
        std::memcpy(static_cast<char*>(_result.dataOffset(_pos)), static_cast<const char*>(data.data()), data.size());
        _pos += data.size();
    }

    inline void Serializer::writeString(const std::string &value) {
        size_t size = value.size() + 1; // +1 for the nullbyte
        if (_pos + size > _result.size()) {
            throw std::runtime_error("Serialization failed - size overflow");
        }
        std::memcpy(static_cast<char*>(_result.dataOffset(_pos)), value.c_str(), size);
        _pos += size;
    }

    inline size_t Serializer::StringSize(const std::string &value) {
        return value.size() + 1; // +1 for nullbyte
    }

    inline Data Serializer::finished() {
        if (_pos != _result.size()) {
            throw std::runtime_error("Serialization failed - size not fully used.");
        }
        return std::move(_result);
    }
}

#endif
