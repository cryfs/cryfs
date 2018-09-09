#include <gtest/gtest.h>
#include <cpp-utils/system/homedir.h>
#include <cpp-utils/tempfile/TempDir.h>

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
    TempDir fakeHomeDir, fakeAppDataDir;
    FakeHomeDirectoryRAII a(fakeHomeDir.path(), fakeAppDataDir.path());

    EXPECT_EQ(fakeHomeDir.path(), HomeDirectory::get());
    EXPECT_EQ(fakeAppDataDir.path(), HomeDirectory::getXDGDataDir());
}

TEST(HomedirTest, FakeHomeDirectoryResetsHomedirCorrectly) {
    bf::path actualHomeDir = HomeDirectory::get();
    bf::path actualAppDataDir = HomeDirectory::getXDGDataDir();

    {
        TempDir fakeHomeDir, fakeAppDataDir;
        FakeHomeDirectoryRAII a(fakeHomeDir.path(), fakeAppDataDir.path());

        EXPECT_NE(actualHomeDir, HomeDirectory::get());
        EXPECT_NE(actualAppDataDir, HomeDirectory::getXDGDataDir());
    }
    EXPECT_EQ(actualHomeDir, HomeDirectory::get());
    EXPECT_EQ(actualAppDataDir, HomeDirectory::getXDGDataDir());
}

TEST(HomedirTest, FakeTempHomeDirectorySetsHomedirCorrectly) {
    bf::path actualHomeDir = HomeDirectory::get();
    bf::path actualAppDataDir = HomeDirectory::getXDGDataDir();

    FakeTempHomeDirectoryRAII a;

    EXPECT_NE(actualHomeDir, HomeDirectory::get());
    EXPECT_NE(actualAppDataDir, HomeDirectory::getXDGDataDir());
}

TEST(HomedirTest, FakeTempHomeDirectoryResetsHomedirCorrectly) {
    bf::path actualHomeDir = HomeDirectory::get();
    bf::path actualAppDataDir = HomeDirectory::getXDGDataDir();

    {
        FakeTempHomeDirectoryRAII a;

        EXPECT_NE(actualHomeDir, HomeDirectory::get());
        EXPECT_NE(actualAppDataDir, HomeDirectory::getXDGDataDir());
    }
    EXPECT_EQ(actualHomeDir, HomeDirectory::get());
    EXPECT_EQ(actualAppDataDir, HomeDirectory::getXDGDataDir());
}

TEST(HomedirTest, FakeTempHomeDirectoryUsesDifferentDirsForHomedirAndAppdataDir) {
    FakeTempHomeDirectoryRAII a;

    EXPECT_NE(HomeDirectory::get(), HomeDirectory::getXDGDataDir());
}
