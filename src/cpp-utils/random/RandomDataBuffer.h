#pragma once
#ifndef MESSMER_CPPUTILS_RANDOM_RANDOMDATABUFFER_H
#define MESSMER_CPPUTILS_RANDOM_RANDOMDATABUFFER_H

#include "../data/Data.h"
#include "../assert/assert.h"

namespace cpputils {
    //TODO Test
    class RandomDataBuffer final {
    public:
        RandomDataBuffer();

        size_t size() const;

        void get(void *target, size_t bytes);

        void add(const Data& data);

    private:
        size_t _usedUntil;
        Data _data;

        DISALLOW_COPY_AND_ASSIGN(RandomDataBuffer);
    };

    inline RandomDataBuffer::RandomDataBuffer() : _usedUntil(0), _data(0) {
    }

    inline size_t RandomDataBuffer::size() const {
        return _data.size() - _usedUntil;
    }

    inline void RandomDataBuffer::get(void *target, size_t numBytes) {
        ASSERT(size() >= numBytes, "Too many bytes requested. Buffer is smaller.");
        std::memcpy(target, _data.dataOffset(_usedUntil), numBytes);
        _usedUntil += numBytes;
    }

    inline void RandomDataBuffer::add(const Data& newData) {
        // Concatenate old and new random data
        const size_t oldSize = size();
        Data combined(oldSize + newData.size());
        get(combined.data(), oldSize);
        std::memcpy(combined.dataOffset(oldSize), newData.data(), newData.size());
        _data = std::move(combined);
        _usedUntil = 0;
    }

}

#endif
