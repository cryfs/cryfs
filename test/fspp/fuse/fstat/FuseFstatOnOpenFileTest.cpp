#include "testutils/FuseFstatTest.h"

#include "fspp/fs_interface/FuseErrnoException.h"

using ::testing::AtLeast;
using ::testing::Eq;
using ::testing::Throw;
using ::testing::Values;
using ::testing::WithParamInterface;

using cpputils::make_unique_ref;
using cpputils::unique_ref;

using namespace fspp::fuse;

// libfuse 2 only called fgetattr() after create(). libfuse 3 merged it into getattr() and hands us
// the open file's fuse_file_info, so requests that come from an open file reach us through the
// handle and the path never has to be resolved again.
//
// The call used here is lseek(fd, 0, SEEK_END). Plain fstat(2) would not do: the kernel's getattr
// inode operation has no struct file to take a handle from, so it asks by path. lseek(SEEK_END)
// goes through fuse_file_llseek(), which does have the file and sets FATTR_FH. The mount uses
// attr_timeout=0 so the kernel asks us instead of answering out of its attribute cache.
class FuseFstatOnOpenFileTest: public FuseTest {
public:
  const char *FILENAME = "/myfile";

  unique_ref<TempTestFS> MountWithoutAttributeCache() {
    return TestFS({"-o", "attr_timeout=0"});
  }

  unique_ref<OpenFileHandle> OpenFile(const TempTestFS *fs) {
    auto realpath = fs->mountDir() / FILENAME;
    auto fd = make_unique_ref<OpenFileHandle>(realpath.string().c_str(), O_RDONLY);
    EXPECT_GE(fd->fd(), 0) << "Opening file failed";
    return fd;
  }
};

class FuseFstatOnOpenFileDescriptorTest: public FuseFstatOnOpenFileTest, public WithParamInterface<int> {
};
INSTANTIATE_TEST_SUITE_P(FuseFstatOnOpenFileDescriptorTest, FuseFstatOnOpenFileDescriptorTest, Values(0, 1, 10, 1000, 1024*1024*1024));

TEST_P(FuseFstatOnOpenFileDescriptorTest, FstatGoesThroughTheFileDescriptor) {
  ReturnIsFileOnLstat(FILENAME);
  OnOpenReturnFileDescriptor(FILENAME, GetParam());
  //this is the point: fstat() and not lstat() answers, and it gets the descriptor open() returned
  EXPECT_CALL(*fsimpl, fstat(Eq(GetParam()), testing::_))
    .Times(AtLeast(1)).WillRepeatedly(ReturnIsFileFstatWithSize(fspp::num_bytes_t(1024)));

  auto fs = MountWithoutAttributeCache();
  auto fd = OpenFile(fs.get());

  EXPECT_EQ(1024, ::lseek(fd->fd(), 0, SEEK_END));
}

class FuseFstatOnOpenFileSizeTest: public FuseFstatOnOpenFileTest, public WithParamInterface<fspp::num_bytes_t> {
};
INSTANTIATE_TEST_SUITE_P(FuseFstatOnOpenFileSizeTest, FuseFstatOnOpenFileSizeTest, Values(
    fspp::num_bytes_t(0),
    fspp::num_bytes_t(1),
    fspp::num_bytes_t(10),
    fspp::num_bytes_t(1024),
    fspp::num_bytes_t(1024*1024*1024)));

TEST_P(FuseFstatOnOpenFileSizeTest, ReturnedSizeIsCorrect) {
  ReturnIsFileOnLstat(FILENAME);
  OnOpenReturnFileDescriptor(FILENAME, 0);
  EXPECT_CALL(*fsimpl, fstat(Eq(0), testing::_))
    .Times(AtLeast(1)).WillRepeatedly(ReturnIsFileFstatWithSize(GetParam()));

  auto fs = MountWithoutAttributeCache();
  auto fd = OpenFile(fs.get());

  EXPECT_EQ(GetParam().value(), ::lseek(fd->fd(), 0, SEEK_END));
}

class FuseFstatOnOpenFileErrorTest: public FuseFstatOnOpenFileTest, public WithParamInterface<int> {
};
INSTANTIATE_TEST_SUITE_P(FuseFstatOnOpenFileErrorTest, FuseFstatOnOpenFileErrorTest, Values(EACCES, EBADF, EFAULT, ELOOP, ENAMETOOLONG, ENOENT, ENOMEM, ENOTDIR, EOVERFLOW));

TEST_P(FuseFstatOnOpenFileErrorTest, ReturnedErrorIsCorrect) {
  ReturnIsFileOnLstat(FILENAME);
  OnOpenReturnFileDescriptor(FILENAME, 0);
  EXPECT_CALL(*fsimpl, fstat(Eq(0), testing::_))
    .Times(AtLeast(1)).WillRepeatedly(Throw(FuseErrnoException(GetParam())));

  auto fs = MountWithoutAttributeCache();
  auto fd = OpenFile(fs.get());

  errno = 0;
  ASSERT_EQ(-1, ::lseek(fd->fd(), 0, SEEK_END)) << "lseek should have failed";
  EXPECT_EQ(GetParam(), errno);
}
