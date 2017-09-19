#pragma once
#ifndef MESSMER_CPPUTILS_THREAD_THREADPOOLEXECUTOR_H
#define MESSMER_CPPUTILS_THREAD_THREADPOOLEXECUTOR_H

#include "MPMCQueue.h"
#include <folly/Function.h>
#include "LoopThread.h"

namespace cpputils {

/**
 * Use this class to run some work in a thread pool.
 * The calling thread doesn't block but immediately returns.
 */
class ThreadPoolExecutor final {
public:
    ThreadPoolExecutor(size_t numThreads);
    ~ThreadPoolExecutor();

    void execute(folly::Function<void ()> task);

private:
    MPMCQueue<folly::Function<void ()>> _tasks;

    std::vector<LoopThread> _executorThreads;

    std::vector<LoopThread> _createExecutorThreads(size_t numThreads);
    bool _executorThreadIteration();

    DISALLOW_COPY_AND_ASSIGN(ThreadPoolExecutor);
};

}

#endif
