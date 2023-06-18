#include "testutils/FuseFstatTest.h"

#include "fspp/fs_interface/FuseErrnoException.h"

using ::testing::WithParamInterface;
using ::testing::Values;
using ::testing::Eq;
using ::testing::Throw;


using namespace fspp::fuse;

// Cite from FUSE documentation on the fgetattr function:
// "Currently this is only called after the create() method if that is implemented (see above).
//  Later it may be called for invocations of fstat() too."
// So we need to issue a create to get our fstat called.

class FuseFstatErrorTest: public FuseFstatTest, public WithParamInterface<int> {
public:
  /*unique_ref<OpenFileHandle> CreateFileAllowErrors(const TempTestFS *fs, const std::string &filename) {
    auto real_path = fs->mountDir() / filename;
    return make_unique_ref<OpenFileHandle>(real_path.string().c_str(), O_RDWR | O_CREAT, S_IRUSR | S_IWUSR | S_IRGRP | S_IROTH);
  }*/
};
INSTANTIATE_TEST_SUITE_P(FuseFstatErrorTest, FuseFstatErrorTest, Values(EACCES, EBADF, EFAULT, ELOOP, ENAMETOOLONG, ENOENT, ENOMEM, ENOTDIR, EOVERFLOW));

TEST_P(FuseFstatErrorTest, ReturnedErrorCodeIsCorrect) {
  ReturnDoesntExistOnLstat(FILENAME);
  OnCreateAndOpenReturnFileDescriptor(FILENAME, 0);

  EXPECT_CALL(*fsimpl, fstat(Eq(0), testing::_)).Times(1).WillOnce(Throw(FuseErrnoException(GetParam())));

  auto fs = TestFS();

  int error = CreateFileReturnError(fs.get(), FILENAME);
  EXPECT_EQ(GetParam(), error);
}
