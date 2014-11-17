#include "gtest/gtest.h"
#include "gmock/gmock.h"

#include <string>
#include <thread>
#include <csignal>

#include "cryfs_lib/CryDevice.h"
#include "test/testutils/FuseThread.h"

#include "fspp/fuse/Fuse.h"
#include "fspp/impl/FilesystemImpl.h"
#include "test/testutils/TempDir.h"

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
  FuseTest(): crydevice(bf::path("/home/heinzi/cryfstest/root")), fsimpl(&crydevice), mountDir(), fuse(&fsimpl), fuse_thread(&fuse) {
    string dirpath = mountDir.path().native();
    int argc = 3;
    const char *argv[] = {"test", "-f", dirpath.c_str()};

    fuse_thread.start(argc, const_cast<char**>(argv));
  }

  ~FuseTest() {
    fuse_thread.stop();
  }

  //MockFilesystemImpl fsimpl;
    cryfs::CryDevice crydevice;
    FilesystemImpl fsimpl;
  TempDir mountDir;
  Fuse fuse;
  FuseThread fuse_thread;
};

TEST_F(FuseTest, setupAndTearDown) {
  //This test case simply checks whether a filesystem can be setup and teardown without crashing.
  //Since this is done in the fixture, we don't need any additional test code here.
}

TEST_F(FuseTest, openFile) {
  const bf::path filename("/myfile");
  //EXPECT_CALL(fsimpl, openFile(filename, O_RDWR))
  //    .WillOnce(Return(1));
  auto realpath = mountDir.path() / filename;
  printf("Opening %s\n", realpath.c_str());
  fflush(stdout);
  sleep(10);
  int fd = ::open(realpath.c_str(), O_RDWR);
  printf("Descriptor: %d, errno: %d\n", fd, errno);
  fflush(stdout);
  sleep(10);
}
