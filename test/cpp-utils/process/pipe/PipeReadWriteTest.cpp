#include <gtest/gtest.h>

#include <cpp-utils/process/pipe/PipeBuilder.h>
#include <thread>
#include <cpp-utils/data/Data.h>

using cpputils::process::PipeBuilder;
using cpputils::process::PipeReader;
using cpputils::process::PipeWriter;
using cpputils::Data;
using std::unique_ptr;
using std::string;

class PipeReadWriteTest : public ::testing::Test {
public:
    PipeBuilder builder;

    string stringWithSize(size_t size) {
        Data data(size);
        std::memset(data.data(), 'a', data.size());
        return string((char*)data.data(), data.size());
    }
};

TEST_F(PipeReadWriteTest, write_then_read) {
    std::thread writeThread([this]() {
        PipeWriter writer = builder.writer();
        writer.send("Hello");
    });
    writeThread.join();

    PipeReader reader = builder.reader();
    EXPECT_EQ("Hello", reader.receive());
}

TEST_F(PipeReadWriteTest, read_then_write) {
    std::thread writeThread([this]() {
        std::this_thread::sleep_for(std::chrono::milliseconds(500));
        PipeWriter writer = builder.writer();
        writer.send("Hello");
    });

    PipeReader reader = builder.reader();
    EXPECT_EQ("Hello", reader.receive());
    writeThread.join();
}

TEST_F(PipeReadWriteTest, newline_in_message) {
    std::thread writeThread([this]() {
        PipeWriter writer = builder.writer();
        writer.send("Hello\n New line");
    });
    writeThread.join();

    PipeReader reader = builder.reader();
    EXPECT_EQ("Hello\n New line", reader.receive());
}

TEST_F(PipeReadWriteTest, WriteMaximumSize) {
    string str = stringWithSize(PipeReader::MAX_READ_SIZE);
    std::thread writeThread([this, str]() {
        PipeWriter writer = builder.writer();
        writer.send(str);
    });

    PipeReader reader = builder.reader();
    EXPECT_EQ(str, reader.receive());

    writeThread.join();
}

TEST_F(PipeReadWriteTest, WriteLargerThanMaximumSize) {
    string str = stringWithSize(PipeReader::MAX_READ_SIZE+1);
    PipeWriter writer = builder.writer();
    EXPECT_THROW(
      writer.send(str),
      std::runtime_error
    );
}

TEST_F(PipeReadWriteTest, interprocess) {
    pid_t pid = fork();
    if (pid < 0) {
        throw std::runtime_error("fork() failed.");
    }
    if (pid == 0) {
        // We're the child process. Send message.
        builder.closeReader();
        PipeWriter writer = builder.writer();
        writer.send("Hello world");
        exit(0);
    }

    // We're the parent process
    builder.closeWriter();
    PipeReader reader = builder.reader();
    EXPECT_EQ("Hello world", reader.receive());
}
