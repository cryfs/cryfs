#pragma once
#ifndef MESSMER_CPPUTILS_FIBERS_ASYNCTHREADPOOLEXECUTOR_H
#define MESSMER_CPPUTILS_FIBERS_ASYNCTHREADPOOLEXECUTOR_H

#include "../thread/ThreadPoolExecutor.h"
#include <boost/fiber/future/promise.hpp>
#include <type_traits>

// TODO Replace all (reasonable) std::mutex and std::condition_variable in the codebase with fiber variants

namespace cpputils {

/**
 * Use this class to run some work in a thread pool.
 * The calling fiber blocks until the result is present, but the calling thread can continue with different fibers.
 */
class AsyncThreadPoolExecutor final {
public:
    AsyncThreadPoolExecutor(size_t numThreads);

    // Call the given callable in the thread pool, block the calling fiber until the result is ready, and return the result.
    template<class Callable, class = std::enable_if_t<!std::is_void<std::result_of_t<Callable()>>::value>>
    std::result_of_t<Callable()> execute(Callable&& task);

    // Specialization for tasks that return void. This also blocks the calling fiber.
    template<class Callable, class = std::enable_if_t<std::is_void<std::result_of_t<Callable()>>::value>>
    void execute(Callable&& task);

    // Call the given callable, DON'T block the calling fiber but immediately return.
    template<class Callable>
    void executeNonBlocking(Callable&& task);

private:
    ThreadPoolExecutor executor_;

    DISALLOW_COPY_AND_ASSIGN(AsyncThreadPoolExecutor);
};

inline AsyncThreadPoolExecutor::AsyncThreadPoolExecutor(size_t numThreads)
    : executor_(numThreads) {}

template<class Callable, class = std::enable_if_t<!std::is_void<std::result_of_t<Callable()>>::value>>
inline std::result_of_t<Callable()> AsyncThreadPoolExecutor::execute(Callable&& task) {
    boost::fibers::promise<std::result_of_t<Callable()>> promise;
    boost::fibers::future<std::result_of_t<Callable()>> future = promise.get_future();
    executor_.execute([promise = std::move(promise), &task] () mutable {
        try {
            promise.set_value(task());
        } catch (...) {
            promise.set_exception(std::current_exception());
        }
    });
    return future.get();
}

template<class Callable, class = std::enable_if_t<std::is_void<std::result_of_t<Callable()>>::value>>
inline void AsyncThreadPoolExecutor::execute(Callable&& task) {
    execute([&task] () -> bool {
        task();
        return true; // fake a bool return value
    });
}

template<class Callable>
void AsyncThreadPoolExecutor::executeNonBlocking(Callable&& task) {
    executor_.execute(std::move(task));
}

}

#endif
