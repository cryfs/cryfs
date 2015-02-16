#pragma once
#ifndef TEST_TESTUTILS_FUSETEST_H_
#define TEST_TESTUTILS_FUSETEST_H_

#include "google/gtest/gtest.h"
#include "google/gmock/gmock.h"

#include "../../fuse/Filesystem.h"
#include "../../fuse/FuseErrnoException.h"
#include "../../fuse/Fuse.h"

#include <boost/filesystem.hpp>

#include <messmer/tempfile/src/TempDir.h>
#include "FuseThread.h"

#define MOCK_PATH_METHOD1(NAME, RETURNTYPE)                        \
  RETURNTYPE NAME(const boost::filesystem::path &path) override {  \
    return NAME(path.c_str());                                     \
  }                                                                \
  MOCK_METHOD1(NAME, RETURNTYPE(const char*));                     \

#define MOCK_PATH_METHOD2(NAME, RETURNTYPE, PARAM1)                               \
  RETURNTYPE NAME(const boost::filesystem::path &path, PARAM1 param1) override {  \
    return NAME(path.c_str(), param1);                                            \
  }                                                                               \
  MOCK_METHOD2(NAME, RETURNTYPE(const char*, PARAM1));                            \

#define MOCK_PATH_METHOD4(NAME, RETURNTYPE, PARAM1, PARAM2, PARAM3)                                 \
  RETURNTYPE NAME(const boost::filesystem::path &path, PARAM1 p1, PARAM2 p2, PARAM3 p3) override {  \
    return NAME(path.c_str(), p1, p2, p3);                                                          \
  }                                                                                                 \
  MOCK_METHOD4(NAME, RETURNTYPE(const char*, PARAM1, PARAM2, PARAM3));                              \

class MockFilesystem: public fspp::fuse::Filesystem {
public:
  MockFilesystem();
  virtual ~MockFilesystem();

  MOCK_PATH_METHOD2(openFile, int, int);
  MOCK_METHOD1(closeFile, void(int));
  MOCK_PATH_METHOD2(lstat, void, struct ::stat*);
  MOCK_METHOD2(fstat, void(int, struct ::stat*));
  MOCK_PATH_METHOD2(truncate, void, off_t);
  MOCK_METHOD2(ftruncate, void(int, off_t));
  MOCK_METHOD4(read, int(int, void*, size_t, off_t));
  MOCK_METHOD4(write, void(int, const void*, size_t, off_t));
  MOCK_METHOD1(flush, void(int));
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
  const char* FILENAME = "/myfile";

  FuseTest();

  class TempTestFS {
  public:
    TempTestFS(MockFilesystem *fsimpl);
    virtual ~TempTestFS();
  public:
    const boost::filesystem::path &mountDir() const;
  private:
    tempfile::TempDir _mountDir;
    fspp::fuse::Fuse _fuse;
    FuseThread _fuse_thread;
  };

  std::unique_ptr<TempTestFS> TestFS();

  MockFilesystem fsimpl;

  static ::testing::Action<void(const char*, struct ::stat*)> ReturnIsFileWithSize(size_t size);

  //TODO Combine ReturnIsFile and ReturnIsFileFstat. This should be possible in gmock by either (a) using ::testing::Undefined as parameter type or (b) using action macros
  static ::testing::Action<void(const char*, struct ::stat*)> ReturnIsFile;
  static ::testing::Action<void(int, struct ::stat*)> ReturnIsFileFstat;
  static ::testing::Action<void(const char*, struct ::stat*)> ReturnIsDir;
  static ::testing::Action<void(const char*, struct ::stat*)> ReturnDoesntExist;

  void ReturnIsFileOnLstat(const boost::filesystem::path &path);
  void ReturnIsFileOnLstatWithSize(const boost::filesystem::path &path, const size_t size);
  void ReturnIsDirOnLstat(const boost::filesystem::path &path);
  void ReturnDoesntExistOnLstat(const boost::filesystem::path &path);
  void OnOpenReturnFileDescriptor(const char *filename, int descriptor);
  void ReturnIsFileOnFstat(int descriptor);
};

#endif
