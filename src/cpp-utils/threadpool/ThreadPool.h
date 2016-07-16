#pragma once
#ifndef MESSMER_CPPUTILS_THREADPOOL_THREADPOOL_H_
#define MESSMER_CPPUTILS_THREADPOOL_THREADPOOL_H_

#include <cpp-utils/macros.h>
#include <vector>
#include <functional>
#include <future>
#include "cpp-utils/threadpool/impl/ThreadsafeQueue.h"
#include "cpp-utils/threadpool/impl/WorkerThread.h"

namespace cpputils {

    //TODO Test cases

    class ThreadPool final {
    public:
        ThreadPool(unsigned int numThreads);

        template<class Result>
        std::future<Result> run(std::packaged_task<Result ()> task);

    private:
        ThreadsafeQueue<std::packaged_task<void ()>> _tasks;
        std::vector<WorkerThread> _threads;

        DISALLOW_COPY_AND_ASSIGN(ThreadPool);
    };

    template<class Result>
    std::future<Result> ThreadPool::run(std::packaged_task<Result ()> task) {
        auto future = task.get_future();
        std::packaged_task<void ()> taskWrapper([task = std::move(task)] () mutable {
            task();
        });
        _tasks.push(std::move(taskWrapper));
        return future;
    }
}

#endif
