#include <gtest/gtest.h>
#include <cpp-utils/thread/ThreadPoolExecutor.h>
#include <future>

using cpputils::ThreadPoolExecutor;

TEST(ThreadPoolExecutorTest, givenExecutorWithOneThread_whenExecuting_thenExecutes) {
    ThreadPoolExecutor executor(1);
    std::promise<int> promise;
    std::future<int> future = promise.get_future();
    executor.execute([promise = std::move(promise)] () mutable {
        promise.set_value(5);
    });
    int result = future.get();
    EXPECT_EQ(5, result);
}

TEST(ThreadPoolExecutorTest, givenExecutorWithOneThread_whenExecutingTwoDependentTasks_thenReturnsCorrectValue) {
    std::promise<bool> isRunningPromise;
    std::future<bool> isRunningFuture = isRunningPromise.get_future();
    std::promise<int> intermediateValuePromise;
    std::promise<int> finalValuePromise;
    std::future<int> finalValueFuture = finalValuePromise.get_future();

    ThreadPoolExecutor executor(2);
    executor.execute([isRunningPromise = std::move(isRunningPromise),
                      finalValuePromise = std::move(finalValuePromise),
                      intermediateValueFuture = intermediateValuePromise.get_future()] () mutable {
        isRunningPromise.set_value(true);
        // block until we get the intermediate value
        int intermediateValue = intermediateValueFuture.get();
        // supply the final value
        finalValuePromise.set_value(intermediateValue + 1);
    });

    // wait until thread is running
    EXPECT_TRUE(isRunningFuture.get());

    // now the created task is blocked on the intermediate value. Create another task that supplies it.
    executor.execute([intermediateValuePromise = std::move(intermediateValuePromise)] () mutable {
        intermediateValuePromise.set_value(5);
    });

    // wait until the final value is supplied
    int finalValue = finalValueFuture.get();
    EXPECT_EQ(6, finalValue);
}


// TODO more tests...