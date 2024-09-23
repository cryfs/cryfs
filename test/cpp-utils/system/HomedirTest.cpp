#include <boost/filesystem/operations.hpp>
#include <boost/filesystem/path.hpp>
#include <cpp-utils/system/homedir.h>
#include <cpp-utils/tempfile/TempDir.h>
#include <gtest/gtest.h>
#include <iterator>

using cpputils::system::HomeDirectory;
using cpputils::system::FakeHomeDirectoryRAII;
using cpputils::system::FakeTempHomeDirectoryRAII;
using cpputils::TempDir;

namespace bf = boost::filesystem;

TEST(HomedirTest, HomedirExists) {
    EXPECT_TRUE(bf::exists(HomeDirectory::get()));
}

TEST(HomedirTest, AppDataDirIsValid) {
    auto dir = HomeDirectory::getXDGDataDir();
    EXPECT_FALSE(dir.empty());
    EXPECT_GE(std::distance(dir.begin(), dir.end()), 2u); // has at least two components
}

TEST(HomedirTest, FakeHomeDirectorySetsHomedirCorrectly) {
    const TempDir fakeHomeDir;
    const TempDir fakeAppDataDir;
    const FakeHomeDirectoryRAII a(fakeHomeDir.path(), fakeAppDataDir.path());

    EXPECT_EQ(fakeHomeDir.path(), HomeDirectory::get());
    EXPECT_EQ(fakeAppDataDir.path(), HomeDirectory::getXDGDataDir());
}

TEST(HomedirTest, FakeHomeDirectoryResetsHomedirCorrectly) {
    const bf::path actualHomeDir = HomeDirectory::get();
    const bf::path actualAppDataDir = HomeDirectory::getXDGDataDir();

    {
        const TempDir fakeHomeDir;
        const TempDir fakeAppDataDir;
        const FakeHomeDirectoryRAII a(fakeHomeDir.path(), fakeAppDataDir.path());

        EXPECT_NE(actualHomeDir, HomeDirectory::get());
        EXPECT_NE(actualAppDataDir, HomeDirectory::getXDGDataDir());
    }
    EXPECT_EQ(actualHomeDir, HomeDirectory::get());
    EXPECT_EQ(actualAppDataDir, HomeDirectory::getXDGDataDir());
}

TEST(HomedirTest, FakeTempHomeDirectorySetsHomedirCorrectly) {
    const bf::path actualHomeDir = HomeDirectory::get();
    const bf::path actualAppDataDir = HomeDirectory::getXDGDataDir();

    const FakeTempHomeDirectoryRAII a;

    EXPECT_NE(actualHomeDir, HomeDirectory::get());
    EXPECT_NE(actualAppDataDir, HomeDirectory::getXDGDataDir());
}

TEST(HomedirTest, FakeTempHomeDirectoryResetsHomedirCorrectly) {
    const bf::path actualHomeDir = HomeDirectory::get();
    const bf::path actualAppDataDir = HomeDirectory::getXDGDataDir();

    {
        const FakeTempHomeDirectoryRAII a;

        EXPECT_NE(actualHomeDir, HomeDirectory::get());
        EXPECT_NE(actualAppDataDir, HomeDirectory::getXDGDataDir());
    }
    EXPECT_EQ(actualHomeDir, HomeDirectory::get());
    EXPECT_EQ(actualAppDataDir, HomeDirectory::getXDGDataDir());
}

TEST(HomedirTest, FakeTempHomeDirectoryUsesDifferentDirsForHomedirAndAppdataDir) {
    const FakeTempHomeDirectoryRAII a;

    EXPECT_NE(HomeDirectory::get(), HomeDirectory::getXDGDataDir());
}
