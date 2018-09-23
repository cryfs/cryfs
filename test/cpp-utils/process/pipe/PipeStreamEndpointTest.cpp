#include <gtest/gtest.h>

#include <cpp-utils/process/pipe/PipeStreamEndpoint.h>
#include "testutils/TestDescriptor.h"

using cpputils::process::PipeStreamEndpoint;

class PipeStreamEndpointTest : public ::testing::Test {
public:
};

TEST_F(PipeStreamEndpointTest, constructor) {
    TestDescriptor fd;
    PipeStreamEndpoint endpoint(fd.get(), "w");
    EXPECT_NE(nullptr, endpoint.stream());
}
