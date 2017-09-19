#include <gtest/gtest.h>
#include <cpp-utils/fibers/AsyncThreadPoolExecutor.h>

using cpputils::AsyncThreadPoolExecutor;

TEST(AsyncThreadPoolExecutorTest, givenExecutorWithOneThread_whenExecuting_thenReturnsCorrectValue) {
    AsyncThreadPoolExecutor executor(1);
    int result = executor.execute([] () {
        return 5;
    });
    EXPECT_EQ(5, result);
}

TEST(AsyncThreadPoolExecutorTest, givenExecutorWithOneThread_whenExecutingReturningVoid_thenStillBlocks) {
    AsyncThreadPoolExecutor executor(1);
    std::promise<bool> promise;
    executor.execute([future = promise.get_future()] () -> void {
        auto status = future.wait_for(std::chrono::milliseconds(100));
        EXPECT_EQ(std::future_status::timeout, status);
    });
    promise.set_value(true);
}

TEST(AsyncThreadPoolExecutorTest, givenExecutorWithOneThread_whenExecutingNonBlocking_thenDoesntBlocks) {
    AsyncThreadPoolExecutor executor(1);
    std::promise<bool> promise;
    executor.executeNonBlocking([future = promise.get_future()] () mutable {
        EXPECT_TRUE(future.get());
    });
    promise.set_value(true);
}

TEST(AsyncThreadPoolExecutorTest, givenExecutorWithOneThread_whenExecutingNonBlocking_thenExecutes) {
    AsyncThreadPoolExecutor executor(1);
    std::promise<int> promise;
    std::future<int> future = promise.get_future();
    executor.executeNonBlocking([promise = std::move(promise)] () mutable {
        promise.set_value(5);
    });
    int result = future.get();
    EXPECT_EQ(5, result);
}

TEST(AsyncThreadPoolExecutorTest, givenExecutorWithOneThread_whenExecutingTwoDependentTasks_thenReturnsCorrectValue) {
    boost::fibers::promise<bool> isRunningPromise;
    boost::fibers::future<bool> isRunningFuture = isRunningPromise.get_future();
    std::promise<int> intermediateValuePromise;

    boost::fibers::promise<int> finalValuePromise;
    boost::fibers::future<int> finalValueFuture = finalValuePromise.get_future();

    AsyncThreadPoolExecutor executor(2);

    boost::fibers::fiber([&executor,
                                 isRunningPromise = std::move(isRunningPromise),
                                 intermediateValueFuture = intermediateValuePromise.get_future(),
                                 finalValuePromise = std::move(finalValuePromise)] () mutable {
        int finalValue = executor.execute([isRunningPromise = std::move(isRunningPromise),
                                           intermediateValueFuture = std::move(intermediateValueFuture)]() mutable {
            isRunningPromise.set_value(true);
            // block until we get the intermediate value
            int intermediateValue = intermediateValueFuture.get();
            // supply the final value
            return intermediateValue + 1;
        });
        finalValuePromise.set_value(finalValue);
    }).detach();

    // wait until first task is running
    EXPECT_TRUE(isRunningFuture.get());

    // now the created task is blocked on the intermediate value. Create another task that supplies it.
    executor.execute([intermediateValuePromise = std::move(intermediateValuePromise)] () mutable {
        intermediateValuePromise.set_value(5);
    });

    // wait until the final value is supplied
    int finalValue = finalValueFuture.get();
    EXPECT_EQ(6, finalValue);
}

// TODO More tests...
