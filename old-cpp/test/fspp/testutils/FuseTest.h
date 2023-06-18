#pragma once
#ifndef MESSMER_FSPP_TEST_TESTUTILS_FUSETEST_H_
#define MESSMER_FSPP_TEST_TESTUTILS_FUSETEST_H_

#include <gtest/gtest.h>
#include <gmock/gmock.h>

#include "fspp/fuse/Filesystem.h"
#include "fspp/fs_interface/FuseErrnoException.h"
#include "fspp/fuse/Fuse.h"
#include "fspp/fs_interface/Dir.h"

#include <boost/filesystem.hpp>

#include <cpp-utils/tempfile/TempDir.h>
#include "FuseThread.h"

class MockFilesystem: public fspp::fuse::Filesystem {
public:
  MockFilesystem();
  virtual ~MockFilesystem();

  MOCK_METHOD(void, setContext, (fspp::Context&&), (override));
  MOCK_METHOD(int, openFile, (const boost::filesystem::path&, int), (override));
  MOCK_METHOD(void, closeFile, (int), (override));
  MOCK_METHOD(void, lstat, (const boost::filesystem::path&, fspp::fuse::STAT*), (override));
  MOCK_METHOD(void, fstat, (int, fspp::fuse::STAT*), (override));
  MOCK_METHOD(void, truncate, (const boost::filesystem::path&, fspp::num_bytes_t), (override));
  MOCK_METHOD(void, ftruncate, (int, fspp::num_bytes_t), (override));
  MOCK_METHOD(fspp::num_bytes_t, read, (int, void*, fspp::num_bytes_t, fspp::num_bytes_t), (override));
  MOCK_METHOD(void, write, (int, const void*, fspp::num_bytes_t, fspp::num_bytes_t), (override));
  MOCK_METHOD(void, flush, (int), (override));
  MOCK_METHOD(void, fsync, (int), (override));
  MOCK_METHOD(void, fdatasync, (int), (override));
  MOCK_METHOD(void, access, (const boost::filesystem::path&, int), (override));
  MOCK_METHOD(int, createAndOpenFile, (const boost::filesystem::path&, mode_t, uid_t, gid_t), (override));
  MOCK_METHOD(void, mkdir, (const boost::filesystem::path&, mode_t, uid_t, gid_t), (override));
  MOCK_METHOD(void, rmdir, (const boost::filesystem::path&), (override));
  MOCK_METHOD(void, unlink, (const boost::filesystem::path&), (override));
  MOCK_METHOD(void, rename, (const boost::filesystem::path&, const boost::filesystem::path&), (override));
  MOCK_METHOD(std::vector<fspp::Dir::Entry>, readDir, (const boost::filesystem::path &path), (override));
  MOCK_METHOD(void, utimens, (const boost::filesystem::path&, timespec, timespec), (override));
  MOCK_METHOD(void, statfs, (struct statvfs*), (override));
  MOCK_METHOD(void, chmod, (const boost::filesystem::path&, mode_t), (override));
  MOCK_METHOD(void, chown, (const boost::filesystem::path&, uid_t, gid_t), (override));
  MOCK_METHOD(void, createSymlink, (const boost::filesystem::path&, const boost::filesystem::path&, uid_t, gid_t), (override));
  MOCK_METHOD(void, readSymlink, (const boost::filesystem::path&, char*, fspp::num_bytes_t), (override));
};

class FuseTest: public ::testing::Test {
public:
  static constexpr const char* FILENAME = "/myfile";

  FuseTest();

  class TempTestFS {
  public:
    TempTestFS(std::shared_ptr<MockFilesystem> fsimpl, const std::vector<std::string>& fuseOptions = {});
    virtual ~TempTestFS();
  public:
    const boost::filesystem::path &mountDir() const;
  private:
    cpputils::TempDir _mountDir;
    fspp::fuse::Fuse _fuse;
    FuseThread _fuse_thread;
  };

  cpputils::unique_ref<TempTestFS> TestFS(const std::vector<std::string>& fuseOptions = {});

  std::shared_ptr<MockFilesystem> fsimpl;

  const fspp::Context& context() const {
      ASSERT(_context != boost::none, "Context wasn't correctly initialized");
      return *_context;
  }
private:
  boost::optional<fspp::Context> _context;

public:

  //TODO Combine ReturnIsFile and ReturnIsFileFstat. This should be possible in gmock by either (a) using ::testing::Undefined as parameter type or (b) using action macros
  static ::testing::Action<void(const boost::filesystem::path&, fspp::fuse::STAT*)> ReturnIsFile; // NOLINT(cppcoreguidelines-avoid-non-const-global-variables)
  static ::testing::Action<void(const boost::filesystem::path&, fspp::fuse::STAT*)> ReturnIsFileWithSize(fspp::num_bytes_t size); // NOLINT(cppcoreguidelines-avoid-non-const-global-variables)
  static ::testing::Action<void(int, fspp::fuse::STAT*)> ReturnIsFileFstat; // NOLINT(cppcoreguidelines-avoid-non-const-global-variables)
  static ::testing::Action<void(int, fspp::fuse::STAT*)> ReturnIsFileFstatWithSize(fspp::num_bytes_t size); // NOLINT(cppcoreguidelines-avoid-non-const-global-variables)
  static ::testing::Action<void(const boost::filesystem::path&, fspp::fuse::STAT*)> ReturnIsDir; // NOLINT(cppcoreguidelines-avoid-non-const-global-variables)
  static ::testing::Action<void(const boost::filesystem::path&, fspp::fuse::STAT*)> ReturnDoesntExist; // NOLINT(cppcoreguidelines-avoid-non-const-global-variables)

  void ReturnIsFileOnLstat(const boost::filesystem::path &path);
  void ReturnIsFileOnLstatWithSize(const boost::filesystem::path &path, fspp::num_bytes_t size);
  void ReturnIsDirOnLstat(const boost::filesystem::path &path);
  void ReturnDoesntExistOnLstat(const boost::filesystem::path &path);
  void OnOpenReturnFileDescriptor(const char *filename, int descriptor);
  void ReturnIsFileOnFstat(int descriptor);
  void ReturnIsFileOnFstatWithSize(int descriptor, fspp::num_bytes_t size);
};

#endif
