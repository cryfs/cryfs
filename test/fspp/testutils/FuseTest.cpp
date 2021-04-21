#include "FuseTest.h"

using ::testing::Eq;
using ::testing::Return;
using ::testing::Throw;
using ::testing::Action;
using ::testing::Invoke;
using std::make_shared;
using std::shared_ptr;

using cpputils::unique_ref;
using cpputils::make_unique_ref;

namespace bf = boost::filesystem;

using namespace fspp::fuse;

MockFilesystem::MockFilesystem() {}
MockFilesystem::~MockFilesystem() {}

FuseTest::FuseTest(): fsimpl(make_shared<MockFilesystem>()), _context(boost::none) {
  using ::testing::_;
  auto defaultAction = Throw(FuseErrnoException(EIO));
  ON_CALL(*fsimpl, openFile(_,_)).WillByDefault(defaultAction);
  ON_CALL(*fsimpl, closeFile(_)).WillByDefault(defaultAction);
  ON_CALL(*fsimpl, lstat(_,_)).WillByDefault(Throw(FuseErrnoException(ENOENT)));
  ON_CALL(*fsimpl, fstat(_,_)).WillByDefault(Throw(FuseErrnoException(ENOENT)));
  ON_CALL(*fsimpl, truncate(_,_)).WillByDefault(defaultAction);
  ON_CALL(*fsimpl, ftruncate(_,_)).WillByDefault(defaultAction);
  ON_CALL(*fsimpl, read(_,_,_,_)).WillByDefault(defaultAction);
  ON_CALL(*fsimpl, write(_,_,_,_)).WillByDefault(defaultAction);
  ON_CALL(*fsimpl, fsync(_)).WillByDefault(defaultAction);
  ON_CALL(*fsimpl, fdatasync(_)).WillByDefault(defaultAction);
  ON_CALL(*fsimpl, access(_,_)).WillByDefault(defaultAction);
  ON_CALL(*fsimpl, createAndOpenFile(_,_,_,_)).WillByDefault(defaultAction);
  ON_CALL(*fsimpl, mkdir(_,_,_,_)).WillByDefault(defaultAction);
  ON_CALL(*fsimpl, rmdir(_)).WillByDefault(defaultAction);
  ON_CALL(*fsimpl, unlink(_)).WillByDefault(defaultAction);
  ON_CALL(*fsimpl, rename(_,_)).WillByDefault(defaultAction);
  ON_CALL(*fsimpl, readDir(_)).WillByDefault(defaultAction);
  ON_CALL(*fsimpl, utimens(_,_,_)).WillByDefault(defaultAction);
  ON_CALL(*fsimpl, statfs(_)).WillByDefault(Invoke([](struct statvfs *result) {
      ::statvfs("/", result); // As dummy value take the values from the root filesystem
  }));
  ON_CALL(*fsimpl, chmod(_,_)).WillByDefault(defaultAction);
  ON_CALL(*fsimpl, chown(_,_,_)).WillByDefault(defaultAction);
  ON_CALL(*fsimpl, createSymlink(_,_,_,_)).WillByDefault(defaultAction);
  ON_CALL(*fsimpl, readSymlink(_,_,_)).WillByDefault(defaultAction);

  EXPECT_CALL(*fsimpl, access(_,_)).WillRepeatedly(Return());

  ON_CALL(*fsimpl, setContext(_)).WillByDefault(Invoke([this] (fspp::Context context) {
      _context = std::move(context);
  }));

  ReturnIsDirOnLstat("/");
}

unique_ref<FuseTest::TempTestFS> FuseTest::TestFS(const std::vector<std::string>& fuseOptions) {
  return make_unique_ref<TempTestFS>(fsimpl, fuseOptions);
}

FuseTest::TempTestFS::TempTestFS(shared_ptr<MockFilesystem> fsimpl, const std::vector<std::string>& fuseOptions)
 :_mountDir(),
  _fuse([fsimpl] (Fuse*) {return fsimpl;}, []{}, "fusetest", boost::none), _fuse_thread(&_fuse) {

  _fuse_thread.start(_mountDir.path(), fuseOptions);
}

FuseTest::TempTestFS::~TempTestFS() {
  _fuse_thread.stop();
}

const bf::path &FuseTest::TempTestFS::mountDir() const {
  return _mountDir.path();
}

Action<void(const boost::filesystem::path&, fspp::fuse::STAT*)> FuseTest::ReturnIsFileWithSize(fspp::num_bytes_t size) {
  return Invoke([size](const boost::filesystem::path&, fspp::fuse::STAT* result) {
    result->st_mode = S_IFREG | S_IRUSR | S_IRGRP | S_IROTH;
    result->st_nlink = 1;
    result->st_size = size.value();
  });
}

//TODO Combine ReturnIsFile and ReturnIsFileFstat. This should be possible in gmock by either (a) using ::testing::Undefined as parameter type or (b) using action macros
Action<void(const boost::filesystem::path&, fspp::fuse::STAT*)> FuseTest::ReturnIsFile = ReturnIsFileWithSize(fspp::num_bytes_t(0));

Action<void(int, fspp::fuse::STAT*)> FuseTest::ReturnIsFileFstat =
  Invoke([](int, fspp::fuse::STAT* result) {
    result->st_mode = S_IFREG | S_IRUSR | S_IRGRP | S_IROTH;
    result->st_nlink = 1;
  });

Action<void(int, fspp::fuse::STAT*)> FuseTest::ReturnIsFileFstatWithSize(fspp::num_bytes_t size) {
  return Invoke([size](int, struct ::stat *result) {
      result->st_mode = S_IFREG | S_IRUSR | S_IRGRP | S_IROTH;
      result->st_nlink = 1;
      result->st_size = size.value();
  });
}

Action<void(const boost::filesystem::path&, fspp::fuse::STAT*)> FuseTest::ReturnIsDir =
  Invoke([](const boost::filesystem::path&, fspp::fuse::STAT* result) {
    result->st_mode = S_IFDIR | S_IRUSR | S_IRGRP | S_IROTH | S_IXUSR | S_IXGRP | S_IXOTH;
    result->st_nlink = 1;
  });

Action<void(const boost::filesystem::path&, fspp::fuse::STAT*)> FuseTest::ReturnDoesntExist = Throw(fspp::fuse::FuseErrnoException(ENOENT));

void FuseTest::OnOpenReturnFileDescriptor(const char *filename, int descriptor) {
  EXPECT_CALL(*fsimpl, openFile(Eq(filename), testing::_)).Times(1).WillOnce(Return(descriptor));
}

void FuseTest::ReturnIsFileOnLstat(const bf::path &path) {
  EXPECT_CALL(*fsimpl, lstat(Eq(path), ::testing::_)).WillRepeatedly(ReturnIsFile);
}

void FuseTest::ReturnIsFileOnLstatWithSize(const bf::path &path, const fspp::num_bytes_t size) {
  EXPECT_CALL(*fsimpl, lstat(Eq(path), ::testing::_)).WillRepeatedly(ReturnIsFileWithSize(size));
}

void FuseTest::ReturnIsDirOnLstat(const bf::path &path) {
  EXPECT_CALL(*fsimpl, lstat(Eq(path), ::testing::_)).WillRepeatedly(ReturnIsDir);
}

void FuseTest::ReturnDoesntExistOnLstat(const bf::path &path) {
  EXPECT_CALL(*fsimpl, lstat(Eq(path), ::testing::_)).WillRepeatedly(ReturnDoesntExist);
}

void FuseTest::ReturnIsFileOnFstat(int descriptor) {
  EXPECT_CALL(*fsimpl, fstat(descriptor, testing::_)).WillRepeatedly(ReturnIsFileFstat);
}

void FuseTest::ReturnIsFileOnFstatWithSize(int descriptor, fspp::num_bytes_t size) {
  EXPECT_CALL(*fsimpl, fstat(descriptor, testing::_)).WillRepeatedly(ReturnIsFileFstatWithSize(size));
}
