#pragma once
#ifndef MESSMER_CPPUTILS_RANDOM_RANDOMGENERATORTHREAD_H
#define MESSMER_CPPUTILS_RANDOM_RANDOMGENERATORTHREAD_H

#include "LoopThread.h"
#include "ThreadsafeRandomDataBuffer.h"
#include <cryptopp/cryptopp/osrng.h>

namespace cpputils {
    //TODO Test
    class RandomGeneratorThread: public LoopThread {
    public:
        RandomGeneratorThread(ThreadsafeRandomDataBuffer *buffer, size_t minSize, size_t maxSize);
        void loopIteration() override;

    private:
        Data _generateRandomData(size_t size);

        CryptoPP::AutoSeededRandomPool _randomGenerator;
        ThreadsafeRandomDataBuffer *_buffer;
        size_t _minSize;
        size_t _maxSize;

        DISALLOW_COPY_AND_ASSIGN(RandomGeneratorThread);
    };
}

#endif
