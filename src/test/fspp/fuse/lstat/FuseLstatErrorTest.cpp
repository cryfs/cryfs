#include "testutils/FuseLstatTest.h"

#include "fspp/impl/FuseErrnoException.h"

using ::testing::StrEq;
using ::testing::_;
using ::testing::Throw;

using fspp::FuseErrnoException;

class FuseLstatErrorTest: public FuseLstatTest {
public:
  const int ERRCODE1 = EIO;
  const int ERRCODE2 = EACCES;
  const int ERRCODE3 = EBADF;
};

TEST_F(FuseLstatErrorTest, ReturnNoError) {
  EXPECT_CALL(fsimpl, lstat(StrEq(FILENAME), _)).Times(1).WillOnce(ReturnIsFile);
  int retval = LstatPathAllowErrors(FILENAME);
  EXPECT_EQ(retval, 0);
}

TEST_F(FuseLstatErrorTest, ReturnError2) {
  EXPECT_CALL(fsimpl, lstat(StrEq(FILENAME), _)).Times(1).WillOnce(Throw(FuseErrnoException(ERRCODE1)));
  int retval = LstatPathAllowErrors(FILENAME);
  EXPECT_EQ(retval, -1);
  EXPECT_EQ(ERRCODE1, errno);
}

TEST_F(FuseLstatErrorTest, ReturnError3) {
  EXPECT_CALL(fsimpl, lstat(StrEq(FILENAME), _)).Times(1).WillOnce(Throw(FuseErrnoException(ERRCODE2)));
  int retval = LstatPathAllowErrors(FILENAME);
  EXPECT_EQ(retval, -1);
  EXPECT_EQ(ERRCODE2, errno);
}

TEST_F(FuseLstatErrorTest, ReturnError4) {
  EXPECT_CALL(fsimpl, lstat(StrEq(FILENAME), _)).Times(1).WillOnce(Throw(FuseErrnoException(ERRCODE3)));
  int retval = LstatPathAllowErrors(FILENAME);
  EXPECT_EQ(retval, -1);
  EXPECT_EQ(ERRCODE3, errno);
}
