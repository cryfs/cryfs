#pragma once
#ifndef MESSMER_CPPUTILS_RANDOM_RANDOMDATABUFFER_H
#define MESSMER_CPPUTILS_RANDOM_RANDOMDATABUFFER_H

#include "../data/Data.h"
#include "../assert/assert.h"

namespace cpputils {
    //TODO Test
    class RandomDataBuffer {
    public:
        RandomDataBuffer();

        size_t size() const;

        void get(void *target, size_t bytes);

        void add(Data data);

    private:
        size_t _usedUntil;
        Data _data;

        DISALLOW_COPY_AND_ASSIGN(RandomDataBuffer);
    };

    inline RandomDataBuffer::RandomDataBuffer() : _data(0) {
    }

    inline size_t RandomDataBuffer::size() const {
        return _data.size() - _usedUntil;
    }

    inline void RandomDataBuffer::get(void *target, size_t numBytes) {
        ASSERT(size() >= numBytes, "Too many bytes requested. Buffer is smaller.");
        std::memcpy(target, _data.dataOffset(_usedUntil), numBytes);
        _usedUntil += numBytes;
    }

    inline void RandomDataBuffer::add(Data data) {
        // Concatenate old and new random data
        Data newdata(_data.size() + data.size());
        std::memcpy(newdata.data(), _data.data(), _data.size());
        std::memcpy(newdata.dataOffset(_data.size()), data.data(), data.size());
        _data = std::move(newdata);
    }

}

#endif
