#include <atomic>
#include <chrono>
#include <cpp-utils/thread/LeftRight.h>
#include <exception>
#include <gtest/gtest.h>
#include <thread>
#include <type_traits>
#include <vector>

using cpputils::LeftRight;
using std::vector;

TEST(LeftRightTest, givenInt_whenWritingAndReading_thenChangesArePresent) {
  LeftRight<int> obj;

  obj.write([] (auto& obj) {obj = 5;});
  int read = obj.read([] (auto& obj) {return obj;});
  EXPECT_EQ(5, read);

  // check changes are also present in background copy
  obj.write([] (auto&) {}); // this switches to the background copy
  read = obj.read([] (auto& obj) {return obj;});
  EXPECT_EQ(5, read);
}

TEST(LeftRightTest, givenVector_whenWritingAndReading_thenChangesArePresent) {
    LeftRight<vector<int>> obj;

    obj.write([] (auto& obj) {obj.push_back(5);});
    vector<int> read = obj.read([] (auto& obj) {return obj;});
    EXPECT_EQ((vector<int>{5}), read);

    obj.write([] (auto& obj) {obj.push_back(6);});
    read = obj.read([] (auto& obj) {return obj;});
    EXPECT_EQ((vector<int>{5, 6}), read);
}

TEST(LeftRightTest, givenVector_whenWritingReturnsValue_thenValueIsReturned) {
    LeftRight<vector<int>> obj;

    auto a = obj.write([] (auto&) -> int {return 5;});
    static_assert(std::is_same<int, decltype(a)>::value, "");
    EXPECT_EQ(5, a);
}

TEST(LeftRightTest, readsCanBeConcurrent) {
    LeftRight<int> obj;
    std::atomic<int> num_running_readers{0};

    std::thread reader1([&] () {
       obj.read([&] (auto&) {
           ++num_running_readers;
           while(num_running_readers.load() < 2) {}
       });
    });

    std::thread reader2([&] () {
        obj.read([&] (auto&) {
            ++num_running_readers;
            while(num_running_readers.load() < 2) {}
        });
    });

    // the threads only finish after both entered the read function.
    // if LeftRight didn't allow concurrency, this would cause a deadlock.
    reader1.join();
    reader2.join();
}

TEST(LeftRightTest, writesCanBeConcurrentWithReads_readThenWrite) {
    LeftRight<int> obj;
	std::atomic<bool> reader_running{false};
    std::atomic<bool> writer_running{false};

    std::thread reader([&] () {
        obj.read([&] (auto&) {
            reader_running = true;
            while(!writer_running.load()) {}
        });
    });

    std::thread writer([&] () {
        // run read first, write second
        while (!reader_running.load()) {}

        obj.write([&] (auto&) {
            writer_running = true;
        });
    });

    // the threads only finish after both entered the read function.
    // if LeftRight didn't allow concurrency, this would cause a deadlock.
    reader.join();
    writer.join();
}

TEST(LeftRightTest, writesCanBeConcurrentWithReads_writeThenRead) {
    LeftRight<int> obj;
    std::atomic<bool> writer_running{false};
    std::atomic<bool> reader_running{false};

    std::thread writer([&] () {
        obj.read([&] (auto&) {
            writer_running = true;
            while(!reader_running.load()) {}
        });
    });

    std::thread reader([&] () {
        // run write first, read second
        while (!writer_running.load()) {}

        obj.read([&] (auto&) {
            reader_running = true;
        });
    });

    // the threads only finish after both entered the read function.
    // if LeftRight didn't allow concurrency, this would cause a deadlock.
    writer.join();
    reader.join();
}

TEST(LeftRightTest, writesCannotBeConcurrentWithWrites) {
    LeftRight<int> obj;
    std::atomic<bool> first_writer_started{false};
    std::atomic<bool> first_writer_finished{false};

    std::thread writer1([&] () {
        obj.write([&] (auto&) {
            first_writer_started = true;
            std::this_thread::sleep_for(std::chrono::milliseconds(50));
            first_writer_finished = true;
        });
    });

    std::thread writer2([&] () {
        // make sure the other writer runs first
        while (!first_writer_started.load()) {}

        obj.write([&] (auto&) {
            // expect the other writer finished before this one starts
            EXPECT_TRUE(first_writer_finished.load());
        });
    });

    writer1.join();
    writer2.join();
}

namespace {
class MyException : std::exception {};
}

TEST(LeftRightTest, whenReadThrowsException_thenThrowsThrough) {
    const LeftRight<int> obj;

    EXPECT_THROW(
        obj.read([](auto&) {throw MyException();}),
        MyException
    );
}

TEST(LeftRightTest, whenWriteThrowsException_thenThrowsThrough) {
    LeftRight<int> obj;

    EXPECT_THROW(
        obj.write([](auto&) {throw MyException();}),
        MyException
    );
}

TEST(LeftRightTest, givenInt_whenWriteThrowsExceptionOnFirstCall_thenResetsToOldState) {
    LeftRight<int> obj;

    obj.write([](auto& obj) {obj = 5;});

    EXPECT_THROW(
        obj.write([](auto& obj) {
            obj = 6;
            throw MyException();
        }),
        MyException
    );

    // check reading it returns old value
    int read = obj.read([] (auto& obj) {return obj;});
    EXPECT_EQ(5, read);

    // check changes are also present in background copy
    obj.write([] (auto&) {}); // this switches to the background copy
    read = obj.read([] (auto& obj) {return obj;});
    EXPECT_EQ(5, read);
}

// note: each write is executed twice, on the foreground and background copy.
// We need to test a thrown exception in either call is handled correctly.
TEST(LeftRightTest, givenInt_whenWriteThrowsExceptionOnSecondCall_thenKeepsNewState) {
    LeftRight<int> obj;

    obj.write([](auto& obj) {obj = 5;});
    bool write_called = false;

    EXPECT_THROW(
        obj.write([&](auto& obj) {
            obj = 6;
            if (write_called) {
                // this is the second time the write callback is executed
                throw MyException();
            } else {
                write_called = true;
            }
        }),
    MyException
    );

    // check reading it returns new value
    int read = obj.read([] (auto& obj) {return obj;});
    EXPECT_EQ(6, read);

    // check changes are also present in background copy
    obj.write([] (auto&) {}); // this switches to the background copy
    read = obj.read([] (auto& obj) {return obj;});
    EXPECT_EQ(6, read);
}

TEST(LeftRightTest, givenVector_whenWriteThrowsException_thenResetsToOldState) {
    LeftRight<vector<int>> obj;

    obj.write([](auto& obj) {obj.push_back(5);});

    EXPECT_THROW(
            obj.write([](auto& obj) {
                obj.push_back(6);
                throw MyException();
            }),
            MyException
    );

    // check reading it returns old value
    vector<int> read = obj.read([] (auto& obj) {return obj;});
    EXPECT_EQ((vector<int>{5}), read);

    // check changes are also present in background copy
    obj.write([] (auto&) {}); // this switches to the background copy
    read = obj.read([] (auto& obj) {return obj;});
    EXPECT_EQ((vector<int>{5}), read);
}
