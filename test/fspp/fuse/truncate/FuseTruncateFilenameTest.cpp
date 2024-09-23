#include "fspp/fs_interface/Types.h"
#include "testutils/FuseTruncateTest.h"
#include "gmock/gmock.h"
#include <gtest/gtest.h>

using ::testing::Eq;
using ::testing::Return;

class FuseTruncateFilenameTest: public FuseTruncateTest {
};

TEST_F(FuseTruncateFilenameTest, TruncateFile) {
  ReturnIsFileOnLstat("/myfile");
  EXPECT_CALL(*fsimpl, truncate(Eq("/myfile"), testing::_))
    .Times(1).WillOnce(Return());

  TruncateFile("/myfile", fspp::num_bytes_t(0));
}

TEST_F(FuseTruncateFilenameTest, TruncateFileNested) {
  ReturnIsDirOnLstat("/mydir");
  ReturnIsFileOnLstat("/mydir/myfile");
  EXPECT_CALL(*fsimpl, truncate(Eq("/mydir/myfile"), testing::_))
    .Times(1).WillOnce(Return());

  TruncateFile("/mydir/myfile", fspp::num_bytes_t(0));
}

TEST_F(FuseTruncateFilenameTest, TruncateFileNested2) {
  ReturnIsDirOnLstat("/mydir");
  ReturnIsDirOnLstat("/mydir/mydir2");
  ReturnIsFileOnLstat("/mydir/mydir2/myfile");
  EXPECT_CALL(*fsimpl, truncate(Eq("/mydir/mydir2/myfile"), testing::_))
    .Times(1).WillOnce(Return());

  TruncateFile("/mydir/mydir2/myfile", fspp::num_bytes_t(0));
}
