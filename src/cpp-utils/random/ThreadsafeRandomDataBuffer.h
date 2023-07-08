#pragma once
#ifndef MESSMER_CPPUTILS_RANDOM_THREADSAFERANDOMDATABUFFER_H
#define MESSMER_CPPUTILS_RANDOM_THREADSAFERANDOMDATABUFFER_H

#include "../data/Data.h"
#include "../assert/assert.h"
#include "RandomDataBuffer.h"
#include <boost/thread.hpp>

namespace cpputils {
    //TODO Test
    class ThreadsafeRandomDataBuffer final {
    public:
        ThreadsafeRandomDataBuffer();

        size_t size() const;

        void get(void *target, size_t numBytes);

        void add(const Data& data);

        void waitUntilSizeIsLessThan(size_t numBytes);

    private:
        size_t _get(void *target, size_t bytes);

        RandomDataBuffer _buffer;
        mutable boost::mutex _mutex;
        boost::condition_variable _dataAddedCv;
        // _dataGottenCv needs to be boost::condition_variable and not std::condition_variable, because the
        // RandomGeneratorThread calling ThreadsafeRandomDataBuffer::waitUntilSizeIsLessThan() needs the waiting to be
        // interruptible to stop the thread in RandomGeneratorThread::stop() or in the RandomGeneratorThread destructor.
        boost::condition_variable _dataGottenCv;

        DISALLOW_COPY_AND_ASSIGN(ThreadsafeRandomDataBuffer);
    };

    inline ThreadsafeRandomDataBuffer::ThreadsafeRandomDataBuffer(): _buffer(), _mutex(), _dataAddedCv(), _dataGottenCv() {
    }

    inline size_t ThreadsafeRandomDataBuffer::size() const {
        const boost::unique_lock<boost::mutex> lock(_mutex);
        return _buffer.size();
    }

    inline void ThreadsafeRandomDataBuffer::get(void *target, size_t numBytes) {
        size_t alreadyGotten = 0;
        while (alreadyGotten < numBytes) {
            const size_t got = _get(static_cast<uint8_t*>(target)+alreadyGotten, numBytes);
            alreadyGotten += got;
            ASSERT(alreadyGotten <= numBytes, "Got too many bytes");
        }
    }

    inline size_t ThreadsafeRandomDataBuffer::_get(void *target, size_t numBytes) {
        boost::unique_lock<boost::mutex> lock(_mutex);
        _dataAddedCv.wait(lock, [this] {
           return _buffer.size() > 0;
        });
        const size_t gettableBytes = (std::min)(_buffer.size(), numBytes);
        _buffer.get(target, gettableBytes);
        _dataGottenCv.notify_all();
        return gettableBytes;
    }

    inline void ThreadsafeRandomDataBuffer::add(const Data& data) {
        const boost::unique_lock<boost::mutex> lock(_mutex);
        _buffer.add(data);
        _dataAddedCv.notify_all();
    }

    inline void ThreadsafeRandomDataBuffer::waitUntilSizeIsLessThan(size_t numBytes) {
        boost::unique_lock<boost::mutex> lock(_mutex);
        _dataGottenCv.wait(lock, [this, numBytes] {
            return _buffer.size() < numBytes;
        });
    }
}

#endif
