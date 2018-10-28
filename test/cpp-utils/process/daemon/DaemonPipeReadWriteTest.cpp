
#include <gtest/gtest.h>

#include <cpp-utils/process/pipe/PipeBuilder.h>
#include <cpp-utils/process/daemon/PipeFromChild.h>
#include <cpp-utils/process/daemon/PipeToParent.h>
#include <thread>
#include <boost/optional/optional_io.hpp>

using namespace cpputils::process;
using boost::none;

class DaemonPipeReadWriteTest : public ::testing::Test {
public:
    PipeBuilder builder;
};

TEST_F(DaemonPipeReadWriteTest, send_ready) {
    std::thread writeThread([this]() {
        PipeToParent writer(builder.writer());
        writer.notifyReady();
    });
    writeThread.join();

    PipeFromChild reader(builder.reader());
    EXPECT_EQ(none, reader.waitForReadyReturnError());
}

TEST_F(DaemonPipeReadWriteTest, send_error) {
    std::thread writeThread([this]() {
        PipeToParent writer(builder.writer());
        writer.notifyError("Error message");
    });
    writeThread.join();

    PipeFromChild reader(builder.reader());
    EXPECT_EQ("Error message", reader.waitForReadyReturnError().value());
}

TEST_F(DaemonPipeReadWriteTest, send_error_empty) {
    std::thread writeThread([this]() {
        PipeToParent writer(builder.writer());
        writer.notifyError("");
    });
    writeThread.join();

    PipeFromChild reader(builder.reader());
    EXPECT_EQ("", reader.waitForReadyReturnError().value());
}
