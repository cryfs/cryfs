#include "RandomGeneratorThread.h"
#include "config_int.h"
#include "cpp-utils/assert/assert.h"
#include "cpp-utils/data/Data.h"
#include "cpp-utils/random/ThreadsafeRandomDataBuffer.h"
#include <cstddef>
#include <functional>

namespace cpputils {

    RandomGeneratorThread::RandomGeneratorThread(ThreadsafeRandomDataBuffer *buffer, size_t minSize, size_t maxSize)
            : _randomGenerator(),
              _buffer(buffer),
              _minSize(minSize),
              _maxSize(maxSize),
              _thread(std::bind(&RandomGeneratorThread::_loopIteration, this), "RandomGeneratorThread") {
        ASSERT(_maxSize >= _minSize, "Invalid parameters");
    }

    void RandomGeneratorThread::start() {
        return _thread.start();
    }

    bool RandomGeneratorThread::_loopIteration() {
        _buffer->waitUntilSizeIsLessThan(_minSize);
        const size_t neededRandomDataSize = _maxSize - _buffer->size();
        ASSERT(_maxSize > _buffer->size(), "This could theoretically fail if another thread refilled the buffer. But we should be the only refilling thread.");
        const Data randomData = _generateRandomData(neededRandomDataSize);
        _buffer->add(randomData);
        return true; // Run another iteration (don't terminate thread)
    }

    Data RandomGeneratorThread::_generateRandomData(size_t size) {
        Data newRandom(size);
        _randomGenerator.GenerateBlock(static_cast<CryptoPP::byte*>(newRandom.data()), size);
        return newRandom;
    }

}
