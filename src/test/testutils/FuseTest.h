#pragma once
#ifndef TEST_TESTUTILS_FUSETEST_H_
#define TEST_TESTUTILS_FUSETEST_H_

#include "gtest/gtest.h"
#include "gmock/gmock.h"

#include "fspp/impl/Filesystem.h"
#include "fspp/impl/FuseErrnoException.h"
#include "fspp/fuse/Fuse.h"

#include <boost/filesystem.hpp>

#include "TempDir.h"
#include "FuseThread.h"

#define MOCK_PATH_METHOD1(NAME, RETURNTYPE)                        \
  RETURNTYPE NAME(const boost::filesystem::path &path) override {                 \
    return NAME(path.c_str());                                     \
  }                                                                \
  MOCK_METHOD1(NAME, RETURNTYPE(const char*));                     \

#define MOCK_PATH_METHOD2(NAME, RETURNTYPE, PARAM1)                \
  RETURNTYPE NAME(const boost::filesystem::path &path, PARAM1 param1) override {  \
    return NAME(path.c_str(), param1);                             \
  }                                                                \
  MOCK_METHOD2(NAME, RETURNTYPE(const char*, PARAM1));             \

#define MOCK_PATH_METHOD4(NAME, RETURNTYPE, PARAM1, PARAM2, PARAM3)                  \
  RETURNTYPE NAME(const boost::filesystem::path &path, PARAM1 p1, PARAM2 p2, PARAM3 p3) override {  \
    return NAME(path.c_str(), p1, p2, p3);                                           \
  }                                                                                  \
  MOCK_METHOD4(NAME, RETURNTYPE(const char*, PARAM1, PARAM2, PARAM3));               \

class MockFilesystem: public fspp::Filesystem {
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
  void rename(const boost::filesystem::path &from, const boost::filesystem::path &to) override {
    return rename(from.c_str(), to.c_str());
  }
  MOCK_METHOD2(rename, void(const char*, const char*));
  std::unique_ptr<std::vector<std::string>> readDir(const boost::filesystem::path &path) {
    return std::unique_ptr<std::vector<std::string>>(readDir(path.c_str()));
  }
  MOCK_METHOD1(readDir, std::vector<std::string>*(const char*));
  void utimens(const boost::filesystem::path &path, const timespec ts[2]) override {
    return utimens(path.c_str(), ts);
  }
  MOCK_METHOD2(utimens, void(const char*,const timespec[2]));
  MOCK_PATH_METHOD2(statfs, void, struct statvfs*);
};

class FuseTest: public ::testing::Test {
public:
  FuseTest(): fsimpl() {
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
      std::string dirpath = _mountDir.path().native();
      int argc = 3;
      const char *argv[] = {"test", "-f", dirpath.c_str()};

      _fuse_thread.start(argc, const_cast<char**>(argv));
    }

    ~TempTestFS() {
      _fuse_thread.stop();
    }
  public:
    const boost::filesystem::path &mountDir() const {
      return _mountDir.path();
    }
  private:
    TempDir _mountDir;
    fspp::fuse::Fuse _fuse;
    FuseThread _fuse_thread;
  };

  std::unique_ptr<TempTestFS> TestFS() {
    return std::make_unique<TempTestFS>(&fsimpl);
  }

  MockFilesystem fsimpl;

  void ReturnIsFileOnLstat(const bf::path &path) {
    EXPECT_CALL(fsimpl, lstat(::testing::StrEq(path.c_str()), ::testing::_)).WillRepeatedly(
      ::testing::Invoke([](const char*, struct ::stat* result) {
        result->st_mode = S_IFREG | S_IRUSR | S_IRGRP | S_IROTH;
        result->st_nlink = 1;
      }));
  }

  void ReturnIsDirOnLstat(const bf::path &path) {
    EXPECT_CALL(fsimpl, lstat(::testing::StrEq(path.c_str()), ::testing::_)).WillRepeatedly(
      ::testing::Invoke([](const char*, struct ::stat* result) {
        result->st_mode = S_IFDIR | S_IRUSR | S_IRGRP | S_IROTH | S_IXUSR | S_IXGRP | S_IXOTH;
        result->st_nlink = 1;
      }));
  }
};

MATCHER_P(OpenFlagsEq, expectedFlags, "") {
  return expectedFlags == (O_ACCMODE & arg);
}

#endif
