#include "FuseTest.h"

using ::testing::StrEq;
using ::testing::_;
using ::testing::Return;
using ::testing::Throw;
using ::testing::Action;
using ::testing::Invoke;

using std::unique_ptr;
using std::make_unique;

namespace bf = boost::filesystem;

using namespace fspp;

MockFilesystem::MockFilesystem() {}
MockFilesystem::~MockFilesystem() {}

FuseTest::FuseTest(): fsimpl() {
  auto defaultAction = Throw(FuseErrnoException(EIO));
  ON_CALL(fsimpl, openFile(_,_)).WillByDefault(defaultAction);
  ON_CALL(fsimpl, closeFile(_)).WillByDefault(defaultAction);
  ON_CALL(fsimpl, lstat(_,_)).WillByDefault(defaultAction);
  ON_CALL(fsimpl, fstat(_,_)).WillByDefault(defaultAction);
  ON_CALL(fsimpl, truncate(_,_)).WillByDefault(defaultAction);
  ON_CALL(fsimpl, ftruncate(_,_)).WillByDefault(defaultAction);
  ON_CALL(fsimpl, read(_,_,_,_)).WillByDefault(defaultAction);
  ON_CALL(fsimpl, write(_,_,_,_)).WillByDefault(defaultAction);
  ON_CALL(fsimpl, fsync(_)).WillByDefault(defaultAction);
  ON_CALL(fsimpl, fdatasync(_)).WillByDefault(defaultAction);
  ON_CALL(fsimpl, access(_,_)).WillByDefault(defaultAction);
  ON_CALL(fsimpl, createAndOpenFile(_,_)).WillByDefault(defaultAction);
  ON_CALL(fsimpl, mkdir(_, _)).WillByDefault(defaultAction);
  ON_CALL(fsimpl, rmdir(_)).WillByDefault(defaultAction);
  ON_CALL(fsimpl, unlink(_)).WillByDefault(defaultAction);
  ON_CALL(fsimpl, rename(_,_)).WillByDefault(defaultAction);
  ON_CALL(fsimpl, readDir(_)).WillByDefault(defaultAction);
  ON_CALL(fsimpl, utimens(_, _)).WillByDefault(defaultAction);
  ON_CALL(fsimpl, statfs(_, _)).WillByDefault(defaultAction);
}

unique_ptr<FuseTest::TempTestFS> FuseTest::TestFS() {
  return make_unique<TempTestFS>(&fsimpl);
}

FuseTest::TempTestFS::TempTestFS(MockFilesystem *fsimpl): _mountDir(), _fuse(fsimpl), _fuse_thread(&_fuse) {
  std::string dirpath = _mountDir.path().native();
  int argc = 3;
  const char *argv[] = {"test", "-f", dirpath.c_str()};

  _fuse_thread.start(argc, const_cast<char**>(argv));
}

FuseTest::TempTestFS::~TempTestFS() {
  _fuse_thread.stop();
}

const bf::path &FuseTest::TempTestFS::mountDir() const {
  return _mountDir.path();
}

Action<void(const char*, struct ::stat*)> FuseTest::ReturnIsFileWithSize(size_t size) {
  return Invoke([size](const char*, struct ::stat* result) {
    result->st_mode = S_IFREG | S_IRUSR | S_IRGRP | S_IROTH;
    result->st_nlink = 1;
    result->st_size = size;
  });
}

//TODO Combine ReturnIsFile and ReturnIsFileFstat. This should be possible in gmock by either (a) using ::testing::Undefined as parameter type or (b) using action macros
Action<void(const char*, struct ::stat*)> FuseTest::ReturnIsFile = ReturnIsFileWithSize(0);

Action<void(int, struct ::stat*)> FuseTest::ReturnIsFileFstat =
  Invoke([](int, struct ::stat* result) {
    result->st_mode = S_IFREG | S_IRUSR | S_IRGRP | S_IROTH;
    result->st_nlink = 1;
  });

Action<void(const char*, struct ::stat*)> FuseTest::ReturnIsDir =
  Invoke([](const char*, struct ::stat* result) {
    result->st_mode = S_IFDIR | S_IRUSR | S_IRGRP | S_IROTH | S_IXUSR | S_IXGRP | S_IXOTH;
    result->st_nlink = 1;
  });

Action<void(const char*, struct ::stat*)> FuseTest::ReturnDoesntExist = Throw(fspp::FuseErrnoException(ENOENT));

void FuseTest::OnOpenReturnFileDescriptor(const char *filename, int descriptor) {
  EXPECT_CALL(fsimpl, openFile(StrEq(filename), _)).Times(1).WillOnce(Return(descriptor));
}

void FuseTest::ReturnIsFileOnLstat(const bf::path &path) {
  EXPECT_CALL(fsimpl, lstat(::testing::StrEq(path.c_str()), ::testing::_)).WillRepeatedly(ReturnIsFile);
}

void FuseTest::ReturnIsFileOnLstatWithSize(const bf::path &path, const size_t size) {
  EXPECT_CALL(fsimpl, lstat(::testing::StrEq(path.c_str()), ::testing::_)).WillRepeatedly(ReturnIsFileWithSize(size));
}

void FuseTest::ReturnIsDirOnLstat(const bf::path &path) {
  EXPECT_CALL(fsimpl, lstat(::testing::StrEq(path.c_str()), ::testing::_)).WillRepeatedly(ReturnIsDir);
}

void FuseTest::ReturnDoesntExistOnLstat(const bf::path &path) {
  EXPECT_CALL(fsimpl, lstat(::testing::StrEq(path.c_str()), ::testing::_)).WillRepeatedly(ReturnDoesntExist);
}

void FuseTest::ReturnIsFileOnFstat(int descriptor) {
  EXPECT_CALL(fsimpl, fstat(descriptor, _)).WillRepeatedly(ReturnIsFileFstat);
}
