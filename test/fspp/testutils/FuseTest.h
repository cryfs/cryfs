#pragma once
#ifndef MESSMER_FSPP_TEST_TESTUTILS_FUSETEST_H_
#define MESSMER_FSPP_TEST_TESTUTILS_FUSETEST_H_

#include <gtest/gtest.h>
#include <gmock/gmock.h>

#include "fspp/fuse/Filesystem.h"
#include "fspp/fuse/FuseErrnoException.h"
#include "fspp/fuse/Fuse.h"
#include "fspp/fs_interface/Dir.h"

#include <boost/filesystem.hpp>

#include <cpp-utils/tempfile/TempDir.h>
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

#define MOCK_PATH_METHOD3(NAME, RETURNTYPE, PARAM1, PARAM2)                              \
  RETURNTYPE NAME(const boost::filesystem::path &path, PARAM1 p1, PARAM2 p2) override {  \
    return NAME(path.c_str(), p1, p2);                                                   \
  }                                                                                      \
  MOCK_METHOD3(NAME, RETURNTYPE(const char*, PARAM1, PARAM2));                           \

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
  MOCK_METHOD4(read, size_t(int, void*, size_t, off_t));
  MOCK_METHOD4(write, void(int, const void*, size_t, off_t));
  MOCK_METHOD1(flush, void(int));
  MOCK_METHOD1(fsync, void(int));
  MOCK_METHOD1(fdatasync, void(int));
  MOCK_PATH_METHOD2(access, void, int);
  MOCK_PATH_METHOD4(createAndOpenFile, int, mode_t, uid_t, gid_t);
  MOCK_PATH_METHOD4(mkdir, void, mode_t, uid_t, gid_t);
  MOCK_PATH_METHOD1(rmdir, void);
  MOCK_PATH_METHOD1(unlink, void);
  void rename(const boost::filesystem::path &from, const boost::filesystem::path &to) override {
    return rename(from.c_str(), to.c_str());
  }
  MOCK_METHOD2(rename, void(const char*, const char*));
  cpputils::unique_ref<std::vector<fspp::Dir::Entry>> readDir(const boost::filesystem::path &path) override {
    return cpputils::nullcheck(std::unique_ptr<std::vector<fspp::Dir::Entry>>(readDir(path.c_str()))).value();
  }
  MOCK_METHOD1(readDir, std::vector<fspp::Dir::Entry>*(const char*));
  void utimens(const boost::filesystem::path &path, timespec lastAccessTime, timespec lastModificationTime) override {
    return utimens(path.c_str(), lastAccessTime, lastModificationTime);
  }
  MOCK_METHOD3(utimens, void(const char*, timespec, timespec));
  MOCK_PATH_METHOD2(statfs, void, struct statvfs*);
  void createSymlink(const boost::filesystem::path &to, const boost::filesystem::path &from, uid_t uid, gid_t gid) override {
    return createSymlink(to.c_str(), from.c_str(), uid, gid);
  }
  MOCK_PATH_METHOD2(chmod, void, mode_t);
  MOCK_PATH_METHOD3(chown, void, uid_t, gid_t);
  MOCK_METHOD4(createSymlink, void(const char*, const char*, uid_t, gid_t));
  MOCK_PATH_METHOD3(readSymlink, void, char*, size_t);
};

class FuseTest: public ::testing::Test {
public:
  static constexpr const char* FILENAME = "/myfile";

  FuseTest();

  class TempTestFS {
  public:
    TempTestFS(MockFilesystem *fsimpl);
    virtual ~TempTestFS();
  public:
    const boost::filesystem::path &mountDir() const;
  private:
    cpputils::TempDir _mountDir;
    fspp::fuse::Fuse _fuse;
    FuseThread _fuse_thread;
  };

  cpputils::unique_ref<TempTestFS> TestFS();

  MockFilesystem fsimpl;


  //TODO Combine ReturnIsFile and ReturnIsFileFstat. This should be possible in gmock by either (a) using ::testing::Undefined as parameter type or (b) using action macros
  static ::testing::Action<void(const char*, struct ::stat*)> ReturnIsFile;
  static ::testing::Action<void(const char*, struct ::stat*)> ReturnIsFileWithSize(size_t size);
  static ::testing::Action<void(int, struct ::stat*)> ReturnIsFileFstat;
  static ::testing::Action<void(int, struct ::stat*)> ReturnIsFileFstatWithSize(size_t size);
  static ::testing::Action<void(const char*, struct ::stat*)> ReturnIsDir;
  static ::testing::Action<void(const char*, struct ::stat*)> ReturnDoesntExist;

  void ReturnIsFileOnLstat(const boost::filesystem::path &path);
  void ReturnIsFileOnLstatWithSize(const boost::filesystem::path &path, const size_t size);
  void ReturnIsDirOnLstat(const boost::filesystem::path &path);
  void ReturnDoesntExistOnLstat(const boost::filesystem::path &path);
  void OnOpenReturnFileDescriptor(const char *filename, int descriptor);
  void ReturnIsFileOnFstat(int descriptor);
  void ReturnIsFileOnFstatWithSize(int descriptor, const size_t size);
};

#endif
