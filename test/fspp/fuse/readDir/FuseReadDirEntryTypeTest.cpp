#include "testutils/FuseReadDirTest.h"

#include "fspp/fs_interface/Dir.h"

using ::testing::Eq;
using ::testing::Return;

using std::string;
using std::vector;

// The kernel gets an entry's type from the file-type bits our readdir() puts into st_mode, and
// reports it back to readdir(3) as d_type. Everything that walks a directory tree uses it to avoid
// a stat() per entry, and in CryFS every one of those stats is a full path traversal.
//
// This is easy to lose. libfuse enables readdirplus by default because the high level library
// always registers a readdirplus handler, and on libfuse 3.0 to 3.10.2 the branch that handles a
// plain (non-plus) reply to a readdirplus request drops st_mode, so every entry comes back as
// DT_UNKNOWN. CryFS refuses the readdirplus capability for that reason, see fspp/fuse/Capabilities.
class FuseReadDirEntryTypeTest: public FuseReadDirTest {
public:
  vector<std::pair<string, unsigned char>> readEntryTypesOf(vector<fspp::Dir::Entry> entries) {
    ReturnIsDirOnLstat(DIRNAME);
    EXPECT_CALL(*fsimpl, readDir(Eq(DIRNAME))).Times(1).WillOnce(Return(std::move(entries)));
    return ReadDirEntryTypes(DIRNAME);
  }
};

TEST_F(FuseReadDirEntryTypeTest, FileEntriesAreReportedAsFiles) {
  const auto entries = readEntryTypesOf({fspp::Dir::Entry(fspp::Dir::EntryType::FILE, "myfile")});
  ASSERT_EQ(1u, entries.size());
  EXPECT_EQ("myfile", entries[0].first);
  EXPECT_EQ(DT_REG, entries[0].second);
}

TEST_F(FuseReadDirEntryTypeTest, DirEntriesAreReportedAsDirs) {
  const auto entries = readEntryTypesOf({fspp::Dir::Entry(fspp::Dir::EntryType::DIR, "mydir")});
  ASSERT_EQ(1u, entries.size());
  EXPECT_EQ("mydir", entries[0].first);
  EXPECT_EQ(DT_DIR, entries[0].second);
}

TEST_F(FuseReadDirEntryTypeTest, SymlinkEntriesAreReportedAsSymlinks) {
  const auto entries = readEntryTypesOf({fspp::Dir::Entry(fspp::Dir::EntryType::SYMLINK, "mylink")});
  ASSERT_EQ(1u, entries.size());
  EXPECT_EQ("mylink", entries[0].first);
  EXPECT_EQ(DT_LNK, entries[0].second);
}

TEST_F(FuseReadDirEntryTypeTest, MixedEntriesKeepTheirOwnType) {
  const auto entries = readEntryTypesOf({
      fspp::Dir::Entry(fspp::Dir::EntryType::FILE, "myfile"),
      fspp::Dir::Entry(fspp::Dir::EntryType::DIR, "mydir"),
      fspp::Dir::Entry(fspp::Dir::EntryType::SYMLINK, "mylink"),
  });
  ASSERT_EQ(3u, entries.size());
  EXPECT_EQ(std::make_pair(string("myfile"), static_cast<unsigned char>(DT_REG)), entries[0]);
  EXPECT_EQ(std::make_pair(string("mydir"), static_cast<unsigned char>(DT_DIR)), entries[1]);
  EXPECT_EQ(std::make_pair(string("mylink"), static_cast<unsigned char>(DT_LNK)), entries[2]);
}
