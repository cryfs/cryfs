#pragma once
#ifndef MESSMER_CPPUTILS_THREAD_MPMCQUEUE_H
#define MESSMER_CPPUTILS_THREAD_MPMCQUEUE_H

#include "cpp-utils/macros.h"
#include <queue>
#include <boost/thread.hpp>

namespace cpputils {

/**
 * An unbounded blocking multi-producer multi-consumer queue
 */
// TODO Test
template<class Entry>
class MPMCQueue final {
public:
    MPMCQueue();

    void push(Entry&& entry);
    void push(const Entry& entry);
    Entry pop();

    void waitUntilEmpty() const;
private:
    std::queue<Entry> _queue;
    // boost::condition_variable and not std::condition_variable because this needs to be an interruption point
    // when used in LoopThread - otherwise LoopThread can't be stopped while waiting.
    mutable boost::mutex _mutex;
    mutable boost::condition_variable _pushedEntryCV;
    mutable boost::condition_variable _poppedEntryCV;

    DISALLOW_COPY_AND_ASSIGN(MPMCQueue);
};

template<class Entry>
inline MPMCQueue<Entry>::MPMCQueue()
: _queue(), _mutex(), _pushedEntryCV(), _poppedEntryCV() {}

template<class Entry>
inline void MPMCQueue<Entry>::push(Entry&& entry) {
    {
        boost::unique_lock<boost::mutex> lock(_mutex);
        _queue.push(std::move(entry));
    }
    _pushedEntryCV.notify_all();
}

template<class Entry>
inline void MPMCQueue<Entry>::push(const Entry& entry) {
    boost::unique_lock<boost::mutex> lock(_mutex);
    _queue.push(entry);
    lock.unlock();

    _pushedEntryCV.notify_all();
}

template<class Entry>
inline Entry MPMCQueue<Entry>::pop() {
    boost::unique_lock<boost::mutex> lock(_mutex);
    _pushedEntryCV.wait(lock, [this]() { return !_queue.empty(); });
    Entry result = std::move(_queue.front());
    _queue.pop();
    lock.unlock();

    _poppedEntryCV.notify_all();
    return result;
}

template<class Entry>
inline void MPMCQueue<Entry>::waitUntilEmpty() const {
    boost::unique_lock<boost::mutex> lock(_mutex);
    _poppedEntryCV.wait(lock, [this] () {return _queue.empty();});
}

}

#endif
