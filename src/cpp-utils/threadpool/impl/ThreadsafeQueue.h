#pragma once
#ifndef MESSMER_CPPUTILS_THREADPOOL_THREADSAFEQUEUE_H_
#define MESSMER_CPPUTILS_THREADPOOL_THREADSAFEQUEUE_H_

#include <cpp-utils/macros.h>
#include <queue>
#include <boost/thread.hpp>

namespace cpputils {

    template<class Entry>
    class ThreadsafeQueue final {
    public:
        ThreadsafeQueue();

        void push(Entry task);
        Entry waitAndPop();

    private:
        std::queue<Entry> _queue;
        boost::mutex _mutex;
        boost::condition_variable _waitForEntry; // boost::condition_variable, because it has to be interruptible

        DISALLOW_COPY_AND_ASSIGN(ThreadsafeQueue);
    };

    template<class Entry>
    inline ThreadsafeQueue<Entry>::ThreadsafeQueue()
            :_queue(), _mutex() {
    }

    template<class Entry>
    inline void ThreadsafeQueue<Entry>::push(Entry task) {
        boost::unique_lock<boost::mutex> lock(_mutex);
        _queue.push(std::move(task));
        _waitForEntry.notify_one();
    }

    template<class Entry>
    inline Entry ThreadsafeQueue<Entry>::waitAndPop() {
        boost::unique_lock<boost::mutex> lock(_mutex);
        _waitForEntry.wait(lock, [this] {return !_queue.empty();});
        auto result = std::move(_queue.front());
        _queue.pop();
        return result;
    }

}

#endif
