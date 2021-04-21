#include "testutils/FuseFlushTest.h"

using ::testing::Eq;
using ::testing::WithParamInterface;
using ::testing::Values;
using ::testing::Return;

using std::string;

// The fuse behaviour is: For each open(), there will be exactly one call to release().
// Directly before this call to release(), flush() will be called. After flush() returns,
// the ::close() syscall (in the process using the filesystem) returns. So the fuse release() call is
// called asynchronously afterwards. Errors have to be returned in the implementation of flush().

// Citing FUSE spec:
//  1) Flush is called on each close() of a file descriptor.
//  2) Filesystems shouldn't assume that flush will always be called after some writes, or that if will be called at all.
// I can't get these sentences together. For the test cases here, I go with the first one and assume that
// flush() will ALWAYS be called on a file close.

class FuseFlushFileDescriptorTest: public FuseFlushTest, public WithParamInterface<int> {
};
INSTANTIATE_TEST_SUITE_P(FuseFlushFileDescriptorTest, FuseFlushFileDescriptorTest, Values(0, 1, 2, 100, 1024*1024*1024));

TEST_P(FuseFlushFileDescriptorTest, FlushOnCloseFile) {
  ReturnIsFileOnLstat(FILENAME);

  EXPECT_CALL(*fsimpl, openFile(Eq(FILENAME), testing::_)).WillOnce(Return(GetParam()));
  EXPECT_CALL(*fsimpl, flush(Eq(GetParam()))).Times(1);

  OpenAndCloseFile(FILENAME);
}
