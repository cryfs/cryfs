#include <gtest/gtest.h>
#include <gmock/gmock.h>

#include "fspp/impl/FuseOpenFileList.h"

#include <stdexcept>

using cpputils::make_unique_ref;

using namespace fspp;

// fspp::OpenFile is spelled out below, because on Windows the headers pull in <windows.h>, which
// declares a global function named OpenFile, and with the using directive above the bare name is
// ambiguous there.
class MockOpenFile: public fspp::OpenFile {
public:
  MockOpenFile(int fileid_, int flags_): fileid(fileid_), flags(flags_), destructed(false) {}
  int fileid, flags;
  bool destructed;

  ~MockOpenFile() override {destructed = true;}

  MOCK_METHOD(fspp::OpenFile::stat_info, stat, (), (const, override));
  MOCK_METHOD(void, truncate, (fspp::num_bytes_t), (const, override));
  MOCK_METHOD(fspp::num_bytes_t, read, (void*, fspp::num_bytes_t, fspp::num_bytes_t), (const, override));
  MOCK_METHOD(void, write, (const void*, fspp::num_bytes_t, fspp::num_bytes_t), (override));
  MOCK_METHOD(void, flush, (), (override));
  MOCK_METHOD(void, fsync, (), (override));
  MOCK_METHOD(void, fdatasync, (), (override));
};

struct FuseOpenFileListTest: public ::testing::Test {
  static constexpr int FILEID1 = 4;
  static constexpr int FLAGS1 = 5;
  static constexpr int FILEID2 = 6;
  static constexpr int FLAGS2 = 7;
  static constexpr int FILEID3 = 8;
  static constexpr int FLAGS3 = 9;

  FuseOpenFileListTest(): list() {}

  FuseOpenFileList list;

  int open(int fileid, int flags) {
    return list.open(make_unique_ref<MockOpenFile>(fileid, flags));
  }
  int open() {
    return open(FILEID1, FILEID2);
  }
  void check(int id, int fileid, int flags) {
	  list.load(id, [=](fspp::OpenFile* _openFile) {
		  MockOpenFile *openFile = dynamic_cast<MockOpenFile*>(_openFile);
		  EXPECT_EQ(fileid, openFile->fileid);
		  EXPECT_EQ(flags, openFile->flags);
	  });
  }
};

TEST_F(FuseOpenFileListTest, EmptyList1) {
	ASSERT_THROW(list.load(0, [](fspp::OpenFile*) {}), fspp::fuse::FuseErrnoException);
}

TEST_F(FuseOpenFileListTest, EmptyList2) {
	ASSERT_THROW(list.load(3, [](fspp::OpenFile*) {}), fspp::fuse::FuseErrnoException);
}

TEST_F(FuseOpenFileListTest, InvalidId) {
  const int valid_id = open();
  const int invalid_id = valid_id + 1;
  ASSERT_THROW(list.load(invalid_id, [](fspp::OpenFile*) {}), fspp::fuse::FuseErrnoException);
}

TEST_F(FuseOpenFileListTest, Open1AndGet) {
  const int id = open(FILEID1, FLAGS1);
  check(id, FILEID1, FLAGS1);
}

TEST_F(FuseOpenFileListTest, Open2AndGet) {
  const int id1 = open(FILEID1, FLAGS1);
  const int id2 = open(FILEID2, FLAGS2);

  check(id1, FILEID1, FLAGS1);
  check(id2, FILEID2, FLAGS2);
}

TEST_F(FuseOpenFileListTest, Open3AndGet) {
  const int id1 = open(FILEID1, FLAGS1);
  const int id2 = open(FILEID2, FLAGS2);
  const int id3 = open(FILEID3, FLAGS3);

  check(id1, FILEID1, FLAGS1);
  check(id3, FILEID3, FLAGS3);
  check(id2, FILEID2, FLAGS2);
}

//TODO Test case fails. Disabled it. Figure out why and reenable.
/*TEST_F(FuseOpenFileListTest, DestructOnClose) {
  int id = open();

  MockOpenFile *openFile = dynamic_cast<MockOpenFile*>(list.get(id));

  EXPECT_FALSE(openFile->destructed);
  list.close(id);
  EXPECT_TRUE(openFile->destructed);
}*/

TEST_F(FuseOpenFileListTest, GetClosedItemOnEmptyList) {
  const int id = open();

  ASSERT_NO_THROW(list.load(id, [](fspp::OpenFile*) {}));
  list.close(id);
  ASSERT_THROW(list.load(id, [](fspp::OpenFile*) {}), fspp::fuse::FuseErrnoException);
}

TEST_F(FuseOpenFileListTest, GetClosedItemOnNonEmptyList) {
  const int id = open();
  open();

  ASSERT_NO_THROW(list.load(id, [](fspp::OpenFile*) {}));
  list.close(id);
  ASSERT_THROW(list.load(id, [](fspp::OpenFile*) {}), fspp::fuse::FuseErrnoException);
}

TEST_F(FuseOpenFileListTest, CloseOnEmptyList1) {
  ASSERT_THROW(list.close(0), std::out_of_range);
}

TEST_F(FuseOpenFileListTest, CloseOnEmptyList2) {
  ASSERT_THROW(list.close(4), std::out_of_range);
}

TEST_F(FuseOpenFileListTest, RemoveInvalidId) {
  const int valid_id = open();
  const int invalid_id = valid_id + 1;
  ASSERT_THROW(list.close(invalid_id), std::out_of_range);
}
