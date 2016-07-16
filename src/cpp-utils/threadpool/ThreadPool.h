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
        std::future<Result> run(std::function<Result ()> task);

    private:
        ThreadsafeQueue<std::function<void ()>> _tasks;
        std::vector<WorkerThread> _threads;

        DISALLOW_COPY_AND_ASSIGN(ThreadPool);
    };

    template<class Result>
    std::future<Result> ThreadPool::run(std::function<Result ()> task) {
        std::promise<Result> resultPromise;
        _tasks.push([&resultPromise, task] {
            try {
                Result result = task();
                resultPromise.set_value(std::move(result));
            } catch (const std::exception &e) {
                resultPromise.set_exception(e);
            } catch (...) {
                resultPromise.set_exception(std::runtime_error("Unknown exception"));
            }
        });
        return resultPromise.get_future();
    }
}

#endif
