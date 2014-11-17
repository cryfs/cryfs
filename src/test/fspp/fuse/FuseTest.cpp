#include "gtest/gtest.h"
#include "gmock/gmock.h"

#include <string>
#include <thread>
#include <csignal>

#include "fspp/fuse/Fuse.h"
#include "fspp/impl/FilesystemImpl.h"
#include "test/testutils/TempDir.h"
#include "test/testutils/Daemon.h"

using namespace fspp;
using namespace fspp::fuse;
using std::string;
using std::unique_ptr;
using std::vector;
using ::testing::Return;

class MockFilesystemImpl: public FilesystemImpl {
public:
  MockFilesystemImpl(): FilesystemImpl(nullptr) {}

  MOCK_METHOD2(openFile, int(const bf::path&, int));
  MOCK_METHOD1(closeFile, void(int));
  MOCK_METHOD2(lstat, void(const bf::path&, struct ::stat*));
  MOCK_METHOD2(fstat, void(int, struct ::stat*));
  MOCK_METHOD2(truncate, void(const bf::path&, off_t));
  MOCK_METHOD2(ftruncate, void(int, off_t));
  MOCK_METHOD4(read, int(int, void*, size_t, off_t));
  MOCK_METHOD4(write, void(int, const void*, size_t, off_t));
  MOCK_METHOD1(fsync, void(int));
  MOCK_METHOD1(fdatasync, void(int));
  MOCK_METHOD2(access, void(const bf::path&, int));
  MOCK_METHOD2(createAndOpenFile, int(const bf::path&, mode_t));
  MOCK_METHOD2(mkdir, void(const bf::path&, mode_t));
  MOCK_METHOD1(rmdir, void(const bf::path&));
  MOCK_METHOD1(unlink, void(const bf::path&));
  MOCK_METHOD2(rename, void(const bf::path&, const bf::path&));
  unique_ptr<vector<string>> readDir(const bf::path &path) {
    return unique_ptr<vector<string>>(readDirMock(path));
  }
  MOCK_METHOD1(readDirMock, vector<string>*(const bf::path&));
  MOCK_METHOD2(utimens, void(const bf::path&, const timespec[2]));
  MOCK_METHOD2(statfs, void(const bf::path&, struct statvfs*));
};

struct FuseTest: public ::testing::Test {
  FuseTest(): _fuse_process([](){}), fsimpl(), fuse(&fsimpl), mountDir() {
    _fuse_process = Daemon([this] () {
      string dirpath = mountDir.path().native();
      int argc = 3;
      const char *argv[] = {"test", "-f", dirpath.c_str()};
      fuse.run(argc, const_cast<char**>(argv));
    });
    _fuse_process.start();
  }
  ~FuseTest() {
    _fuse_process.stop();
  }

  Daemon _fuse_process;
  MockFilesystemImpl fsimpl;
  Fuse fuse;
  TempDir mountDir;
};

TEST_F(FuseTest, setupAndTearDown) {
  //This test case simply checks whether a filesystem can be setup and teardown without crashing.
  //Since this is done in the fixture, we don't need any additional test code here.
}

TEST_F(FuseTest, openFile) {
  const bf::path filename("/myfile");
  EXPECT_CALL(fsimpl, openFile(filename, O_RDWR))
      .WillOnce(Return(1));

  ::open((mountDir.path() / filename).c_str(), O_RDWR);
}
