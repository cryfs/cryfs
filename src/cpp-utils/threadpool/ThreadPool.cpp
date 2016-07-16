#include "ThreadPool.h"

namespace cpputils {

    ThreadPool::ThreadPool(unsigned int numThreads)
        : _tasks(), _threads() {
        _threads.reserve(numThreads);
        for (unsigned int i = 0; i < numThreads; ++i) {
            _threads.emplace_back(&_tasks);
        }
    }

}
