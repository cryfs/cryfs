#include "testutils/FuseCreateAndOpenTest.h"

//TODO Disabled because it doesn't seem to work. Fuse doesn't seem to pass flags to create(). Why?
/*

using ::testing::WithParamInterface;
using ::testing::Values;

class FuseCreateAndOpenFlagsTest: public FuseCreateAndOpenTest, public WithParamInterface<mode_t> {
};
INSTANTIATE_TEST_SUITE_P(FuseCreateAndOpenFlagsTest, FuseCreateAndOpenFlagsTest, Values(O_RDWR, O_RDONLY, O_WRONLY));

TEST_P(FuseCreateAndOpenFlagsTest, testFlags) {
  ReturnDoesntExistOnLstat(FILENAME);
  EXPECT_CALL(*fsimpl, createAndOpenFile(Eq(FILENAME), OpenFlagsEq(GetParam()), _, _))
    .Times(1).WillOnce(Return(0));
  //For the syscall to succeed, we also need to give an fstat implementation.
  ReturnIsFileOnFstat(0);

  CreateAndOpenFile(FILENAME, GetParam());
}*/
