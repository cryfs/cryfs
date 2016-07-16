#pragma once
#ifndef MESSMER_CPPUTILS_THREADPOOL_THREADSAFEQUEUE_H_
#define MESSMER_CPPUTILS_THREADPOOL_THREADSAFEQUEUE_H_

#include <cpp-utils/macros.h>
#include <queue>
#include <mutex>
#include <condition_variable>

namespace cpputils {

    template<class Entry>
    class ThreadsafeQueue final {
    public:
        ThreadsafeQueue();

        void push(Entry task);
        Entry waitAndPop();

    private:
        std::queue<Entry> _queue;
        std::mutex _mutex;
        std::condition_variable _waitForEntry;

        DISALLOW_COPY_AND_ASSIGN(ThreadsafeQueue);
    };

    template<class Entry>
    inline ThreadsafeQueue<Entry>::ThreadsafeQueue()
            :_queue(), _mutex() {
    }

    template<class Entry>
    inline void ThreadsafeQueue<Entry>::push(Entry task) {
        std::lock_guard<std::mutex> lock(_mutex);
        _queue.push(task);
        _waitForEntry.notify_one();
    }

    template<class Entry>
    inline Entry ThreadsafeQueue<Entry>::waitAndPop() {
        std::unique_lock<std::mutex> lock(_mutex);
        _waitForEntry.wait(lock, [this] {return !_queue.empty();});
        auto result = _queue.front();
        _queue.pop();
        return result;
    }

}

#endif
