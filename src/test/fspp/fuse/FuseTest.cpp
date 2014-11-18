#include "gtest/gtest.h"
#include "gmock/gmock.h"

#include <string>
#include <thread>
#include <csignal>
#include <fcntl.h>

#include "cryfs_lib/CryDevice.h"
#include "test/testutils/FuseThread.h"

#include "fspp/fuse/Fuse.h"
#include "fspp/impl/Filesystem.h"
#include "fspp/impl/FuseErrnoException.h"
#include "test/testutils/TempDir.h"

using namespace fspp;
using namespace fspp::fuse;
using std::string;
using std::unique_ptr;
using std::make_unique;
using std::vector;
using ::testing::Return;
using ::testing::_;
using ::testing::Invoke;
using ::testing::Throw;
using ::testing::NiceMock;
using ::testing::StrictMock;
using ::testing::AtMost;
using ::testing::Mock;
using ::testing::StrEq;

#define MOCK_PATH_METHOD1(NAME, RETURNTYPE)                        \
  RETURNTYPE NAME(const bf::path &path) override {                 \
    return NAME(path.c_str());                                     \
  }                                                                \
  MOCK_METHOD1(NAME, RETURNTYPE(const char*));                     \

#define MOCK_PATH_METHOD2(NAME, RETURNTYPE, PARAM1)                \
  RETURNTYPE NAME(const bf::path &path, PARAM1 param1) override {  \
    return NAME(path.c_str(), param1);                             \
  }                                                                \
  MOCK_METHOD2(NAME, RETURNTYPE(const char*, PARAM1));             \

#define MOCK_PATH_METHOD4(NAME, RETURNTYPE, PARAM1, PARAM2, PARAM3)                  \
  RETURNTYPE NAME(const bf::path &path, PARAM1 p1, PARAM2 p2, PARAM3 p3) override {  \
    return NAME(path.c_str(), p1, p2, p3);                                           \
  }                                                                                  \
  MOCK_METHOD4(NAME, RETURNTYPE(const char*, PARAM1, PARAM2, PARAM3));               \

class MockFilesystem: public Filesystem {
public:
  MOCK_PATH_METHOD2(openFile, int, int);
  MOCK_METHOD1(closeFile, void(int));
  MOCK_PATH_METHOD2(lstat, void, struct ::stat*);
  MOCK_METHOD2(fstat, void(int, struct ::stat*));
  MOCK_PATH_METHOD2(truncate, void, off_t);
  MOCK_METHOD2(ftruncate, void(int, off_t));
  MOCK_METHOD4(read, int(int, void*, size_t, off_t));
  MOCK_METHOD4(write, void(int, const void*, size_t, off_t));
  MOCK_METHOD1(fsync, void(int));
  MOCK_METHOD1(fdatasync, void(int));
  MOCK_PATH_METHOD2(access, void, int);
  MOCK_PATH_METHOD2(createAndOpenFile, int, mode_t);
  MOCK_PATH_METHOD2(mkdir, void, mode_t);
  MOCK_PATH_METHOD1(rmdir, void);
  MOCK_PATH_METHOD1(unlink, void);
  void rename(const bf::path &from, const bf::path &to) override {
    return rename(from.c_str(), to.c_str());
  }
  MOCK_METHOD2(rename, void(const char*, const char*));
  unique_ptr<vector<string>> readDir(const bf::path &path) {
    return unique_ptr<vector<string>>(readDir(path.c_str()));
  }
  MOCK_METHOD1(readDir, vector<string>*(const char*));
  void utimens(const bf::path &path, const timespec ts[2]) override {
    return utimens(path.c_str(), ts);
  }
  MOCK_METHOD2(utimens, void(const char*,const timespec[2]));
  MOCK_PATH_METHOD2(statfs, void, struct statvfs*);
};

struct FuseTest: public ::testing::Test {
  FuseTest(): fsimpl() {}

  void SetUp() override {
    /*auto defaultAction = Throw(fspp::FuseErrnoException(EIO));
    ON_CALL(fsimpl, openFile(_,_)).WillByDefault(defaultAction);
    ON_CALL(fsimpl, closeFile(_)).WillByDefault(defaultAction);
    ON_CALL(fsimpl, lstat(_,_)).WillByDefault(Invoke([](const char *path, struct ::stat *) {
      printf("LSTAT\n");fflush(stdout);
      throw fspp::FuseErrnoException(EIO);
    }));
    ON_CALL(fsimpl, fstat(_,_)).WillByDefault(defaultAction);
    ON_CALL(fsimpl, truncate(_,_)).WillByDefault(defaultAction);
    ON_CALL(fsimpl, ftruncate(_,_)).WillByDefault(defaultAction);
    ON_CALL(fsimpl, read(_,_,_,_)).WillByDefault(defaultAction);
    ON_CALL(fsimpl, write(_,_,_,_)).WillByDefault(defaultAction);
    ON_CALL(fsimpl, fsync(_)).WillByDefault(defaultAction);
    ON_CALL(fsimpl, fdatasync(_)).WillByDefault(defaultAction);
    ON_CALL(fsimpl, access(_, _)).WillByDefault(defaultAction);
    ON_CALL(fsimpl, createAndOpenFile(_,_)).WillByDefault(defaultAction);
    ON_CALL(fsimpl, mkdir(_, _)).WillByDefault(defaultAction);
    ON_CALL(fsimpl, rmdir(_)).WillByDefault(defaultAction);
    ON_CALL(fsimpl, unlink(_)).WillByDefault(defaultAction);
    ON_CALL(fsimpl, rename(_, _)).WillByDefault(defaultAction);
    ON_CALL(fsimpl, readDir(_)).WillByDefault(defaultAction);
    ON_CALL(fsimpl, utimens(_, _)).WillByDefault(defaultAction);
    ON_CALL(fsimpl, statfs(_, _)).WillByDefault(defaultAction);*/
  }

  class TempTestFS {
  public:
    TempTestFS(MockFilesystem *fsimpl): _mountDir(), _fuse(fsimpl), _fuse_thread(&_fuse) {
      string dirpath = _mountDir.path().native();
      int argc = 3;
      const char *argv[] = {"test", "-f", dirpath.c_str()};

      _fuse_thread.start(argc, const_cast<char**>(argv));
    }

    ~TempTestFS() {
      _fuse_thread.stop();
    }
  public:
    const bf::path &mountDir() const {
      return _mountDir.path();
    }
  private:
    TempDir _mountDir;
    Fuse _fuse;
    FuseThread _fuse_thread;
  };

  unique_ptr<TempTestFS> TestFS() {
    return make_unique<TempTestFS>(&fsimpl);
  }

  MockFilesystem fsimpl;
};

TEST_F(FuseTest, setupAndTearDown) {
  //This test case simply checks whether a filesystem can be setup and teardown without crashing.
  auto fs = TestFS();
}

TEST_F(FuseTest, openFile) {
  const char *filename = "/myfile";
  EXPECT_CALL(fsimpl, lstat(StrEq(filename), _))
      .WillOnce(Invoke([](const char*, struct ::stat* result) {
    result->st_mode = S_IFREG;
  }));
  EXPECT_CALL(fsimpl, openFile(StrEq(filename), _))
      .WillOnce(Invoke([](const char*, int flags) {
    EXPECT_EQ(O_RDWR, O_ACCMODE & flags);
    return 0;
  }));

  auto fs = TestFS();

  auto realpath = fs->mountDir() / filename;
  ::open(realpath.c_str(), O_RDWR);
}
