#include <gtest/gtest.h>

#include <cpp-utils/process/pipe/PipeDescriptor.h>
#include "testutils/TestDescriptor.h"

using cpputils::process::PipeDescriptor;

class PipeDescriptorTest : public ::testing::Test {
public:

    void EXPECT_IS_NOT_CLOSED(int fd) {
        EXPECT_NE(-1, fcntl(fd, F_GETFD));
    }

    void EXPECT_IS_CLOSED(int fd) {
        EXPECT_EQ(-1, fcntl(fd, F_GETFD));
        EXPECT_EQ(EBADF, errno);
    }

    void EXPECT_CANNOT_BE_CLOSED(PipeDescriptor &desc) {
        EXPECT_THROW(
            desc.close(),
            std::logic_error
        );
    }

    void EXPECT_CAN_BE_CLOSED(PipeDescriptor &desc) {
        int fd = desc.fd();
        EXPECT_IS_NOT_CLOSED(fd);
        desc.close(); // Test it doesn't throw
        EXPECT_IS_CLOSED(fd);
    }

};

TEST_F(PipeDescriptorTest, valid_defaultconstructor) {
    PipeDescriptor desc;
    EXPECT_FALSE(desc.valid());
}

TEST_F(PipeDescriptorTest, valid_constructor) {
    TestDescriptor fd;
    PipeDescriptor desc(fd.get());
    EXPECT_TRUE(desc.valid());
}

TEST_F(PipeDescriptorTest, valid_moveconstructor) {
    TestDescriptor fd;
    PipeDescriptor desc1(fd.get());
    PipeDescriptor desc2(std::move(desc1));
    EXPECT_FALSE(desc1.valid());
    EXPECT_TRUE(desc2.valid());
}

TEST_F(PipeDescriptorTest, valid_moveassignment) {
    TestDescriptor fd;
    PipeDescriptor desc1(fd.get());
    PipeDescriptor desc2;
    desc2 = std::move(desc1);
    EXPECT_FALSE(desc1.valid());
    EXPECT_TRUE(desc2.valid());
}

TEST_F(PipeDescriptorTest, close) {
    TestDescriptor fd;
    PipeDescriptor desc(fd.get());
    EXPECT_IS_NOT_CLOSED(fd.get());
    desc.close();
    EXPECT_IS_CLOSED(fd.get());
}

TEST_F(PipeDescriptorTest, close_defaultconstructor) {
    PipeDescriptor desc;
    EXPECT_CANNOT_BE_CLOSED(desc);
}

TEST_F(PipeDescriptorTest, close_constructor) {
    TestDescriptor fd;
    PipeDescriptor desc(fd.get());
    EXPECT_CAN_BE_CLOSED(desc);
}

TEST_F(PipeDescriptorTest, close_moveconstructor) {
    TestDescriptor fd;
    PipeDescriptor desc1(fd.get());
    PipeDescriptor desc2(std::move(desc1));
    EXPECT_CANNOT_BE_CLOSED(desc1);
    EXPECT_CAN_BE_CLOSED(desc2);
}

TEST_F(PipeDescriptorTest, close_moveassignment) {
    TestDescriptor fd;
    PipeDescriptor desc1(fd.get());
    PipeDescriptor desc2;
    desc2 = std::move(desc1);
    EXPECT_CANNOT_BE_CLOSED(desc1);
    EXPECT_CAN_BE_CLOSED(desc2);
}

TEST_F(PipeDescriptorTest, destructor_closes) {
    TestDescriptor fd;
    {
        PipeDescriptor desc(fd.get());
        EXPECT_IS_NOT_CLOSED(fd.get());
    }
    EXPECT_IS_CLOSED(fd.get());
}
