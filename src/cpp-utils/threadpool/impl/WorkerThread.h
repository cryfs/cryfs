#pragma once
#ifndef MESSMER_CPPUTILS_THREADPOOL_WORKERTHREAD_H_
#define MESSMER_CPPUTILS_THREADPOOL_WORKERTHREAD_H_

#include <cpp-utils/macros.h>
#include <functional>
#include "cpp-utils/thread/LoopThread.h"
#include "ThreadsafeQueue.h"

namespace cpputils {

    class WorkerThread final {
    public:
        WorkerThread(ThreadsafeQueue<std::function<void ()>> *taskQueue);
        WorkerThread(WorkerThread &&rhs) = default;

    private:
        ThreadsafeQueue<std::function<void ()>> *_taskQueue;
        LoopThread _thread;

        bool _loopIteration();

        DISALLOW_COPY_AND_ASSIGN(WorkerThread);
    };

}


#endif
