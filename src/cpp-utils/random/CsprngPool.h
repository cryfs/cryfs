#pragma once
#ifndef MESSMER_CPPUTILS_RANDOM_CSPRNGPOOL_H
#define MESSMER_CPPUTILS_RANDOM_CSPRNGPOOL_H

#include "RandomGenerator.h"
#include "RandomGeneratorThread.h"
#include "ThreadsafeRandomDataBuffer.h"
#include <boost/thread.hpp>
#include <cstddef>
#include <mutex>

namespace cpputils {
    //TODO Test
    class CsprngPool final : public RandomGenerator {
    public:
        CsprngPool();

    protected:
        void _get(void *target, size_t bytes) override;

    private:
        static constexpr size_t MIN_BUFFER_SIZE = 1*1024*1024; // 1MB
        static constexpr size_t MAX_BUFFER_SIZE = 2*1024*1024; // 2MB

        ThreadsafeRandomDataBuffer _buffer;
        RandomGeneratorThread _refillThread;
        DISALLOW_COPY_AND_ASSIGN(CsprngPool);
    };


    inline void CsprngPool::_get(void *target, size_t bytes) {
        _buffer.get(target, bytes);
    }

    inline CsprngPool::CsprngPool(): _buffer(), _refillThread(&_buffer, MIN_BUFFER_SIZE, MAX_BUFFER_SIZE) {
        _refillThread.start();
    }
}

#endif
