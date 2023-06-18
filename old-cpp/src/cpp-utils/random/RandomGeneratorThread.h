#pragma once
#ifndef MESSMER_CPPUTILS_RANDOM_RANDOMGENERATORTHREAD_H
#define MESSMER_CPPUTILS_RANDOM_RANDOMGENERATORTHREAD_H

#include "../thread/LoopThread.h"
#include "ThreadsafeRandomDataBuffer.h"
#include <vendor_cryptopp/osrng.h>

namespace cpputils {
    //TODO Test
    class RandomGeneratorThread final {
    public:
        RandomGeneratorThread(ThreadsafeRandomDataBuffer *buffer, size_t minSize, size_t maxSize);

        void start();

    private:
        bool _loopIteration();
        Data _generateRandomData(size_t size);

        CryptoPP::AutoSeededRandomPool _randomGenerator;
        ThreadsafeRandomDataBuffer *_buffer;
        size_t _minSize;
        size_t _maxSize;

        //This has to be the last member, because it has to be destructed first - otherwise the thread could still be
        //running while the RandomGeneratorThread object is invalid.
        LoopThread _thread;

        DISALLOW_COPY_AND_ASSIGN(RandomGeneratorThread);
    };
}

#endif
