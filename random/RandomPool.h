#pragma once
#ifndef MESSMER_CPPUTILS_RANDOM_RANDOMPOOL_H
#define MESSMER_CPPUTILS_RANDOM_RANDOMPOOL_H

#include <boost/thread.hpp>
#include "ThreadsafeRandomDataBuffer.h"
#include "RandomGeneratorThread.h"
#include <mutex>

namespace cpputils {
    //TODO Test
    class RandomPool final {
    public:
        static void get(void *target, size_t bytes);

    private:
        static constexpr size_t MIN_BUFFER_SIZE = 1*1024*1024; // 1MB
        static constexpr size_t MAX_BUFFER_SIZE = 2*1024*1024; // 2MB

        RandomPool();
        static RandomPool &singleton();
        static std::mutex _mutex;

        ThreadsafeRandomDataBuffer _buffer;
        RandomGeneratorThread _refillThread;
        DISALLOW_COPY_AND_ASSIGN(RandomPool);
    };

    inline RandomPool &RandomPool::singleton() {
        std::unique_lock<std::mutex> lock(_mutex);
        static RandomPool singleton;
        return singleton;
    }

    inline void RandomPool::get(void *target, size_t bytes) {
        singleton()._buffer.get(target, bytes);
    }

    inline RandomPool::RandomPool(): _buffer(), _refillThread(&_buffer, MIN_BUFFER_SIZE, MAX_BUFFER_SIZE) {
        _refillThread.start();
    }
}

#endif
