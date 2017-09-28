#include <gtest/gtest.h>

#include <cryfs/localstate/BasedirMetadata.h>
#include <cryfs/localstate/LocalStateDir.h>
#include <cryfs/config/CryConfig.h>
#include <cpp-utils/tempfile/TempDir.h>

using cpputils::TempDir;
using cryfs::BasedirMetadata;
using std::ofstream;
namespace bf = boost::filesystem;
using FilesystemID = cryfs::CryConfig::FilesystemID ;

class BasedirMetadataTest : public ::testing::Test {
public:
    TempDir tempdir;
    bf::path basedir1;
    bf::path basedir2;
    const FilesystemID id1;
    const FilesystemID id2;

  BasedirMetadataTest()
      : tempdir()
      , basedir1(tempdir.path() / "my/basedir")
      , basedir2(tempdir.path() / "my/other/basedir")
      , id1(FilesystemID::FromString("1491BB4932A389EE14BC7090AC772972"))
      , id2(FilesystemID::FromString("A1491BB493214BC7090C772972A389EE"))
  {
    // Use temporary local state dir to not pollute local state
    cryfs::LocalStateDir::setAppDir(tempdir.path() / "appdir");
    // Create basedirs so bf::canonical() works
    bf::create_directories(basedir1);
    bf::create_directories(basedir2);
  }

};

TEST_F(BasedirMetadataTest, givenEmptyState_whenCalled_thenSucceeds) {
  EXPECT_TRUE(BasedirMetadata::filesystemIdForBasedirIsCorrect(basedir1, id1));
}

TEST_F(BasedirMetadataTest, givenStateWithBasedir_whenCalledForDifferentBasedir_thenSucceeds) {
  BasedirMetadata::updateFilesystemIdForBasedir(basedir2, id1);
  EXPECT_TRUE(BasedirMetadata::filesystemIdForBasedirIsCorrect(basedir1, id1));
}

TEST_F(BasedirMetadataTest, givenStateWithBasedir_whenCalledWithSameId_thenSucceeds) {
  BasedirMetadata::updateFilesystemIdForBasedir(basedir1, id1);
  EXPECT_TRUE(BasedirMetadata::filesystemIdForBasedirIsCorrect(basedir1, id1));
}

TEST_F(BasedirMetadataTest, givenStateWithBasedir_whenCalledWithDifferentId_thenFails) {
  BasedirMetadata::updateFilesystemIdForBasedir(basedir1, id2);
  EXPECT_FALSE(BasedirMetadata::filesystemIdForBasedirIsCorrect(basedir1, id1));
}

TEST_F(BasedirMetadataTest, givenStateWithUpdatedBasedir_whenCalledWithSameId_thenSucceeds) {
  BasedirMetadata::updateFilesystemIdForBasedir(basedir1, id2);
  BasedirMetadata::updateFilesystemIdForBasedir(basedir1, id1);
  EXPECT_TRUE(BasedirMetadata::filesystemIdForBasedirIsCorrect(basedir1, id1));
}

TEST_F(BasedirMetadataTest, givenStateWithUpdatedBasedir_whenCalledWithDifferentId_thenFails) {
  BasedirMetadata::updateFilesystemIdForBasedir(basedir1, id2);
  BasedirMetadata::updateFilesystemIdForBasedir(basedir1, id1);
  EXPECT_FALSE(BasedirMetadata::filesystemIdForBasedirIsCorrect(basedir1, id2));
}
