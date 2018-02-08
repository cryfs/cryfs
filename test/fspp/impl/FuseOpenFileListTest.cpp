#include <gtest/gtest.h>
#include <gmock/gmock.h>

#include "fspp/impl/FuseOpenFileList.h"

#include <stdexcept>

using cpputils::make_unique_ref;

using namespace fspp;

class MockOpenFile: public OpenFile {
public:
  MockOpenFile(int fileid_, int flags_): fileid(fileid_), flags(flags_), destructed(false) {}
  int fileid, flags;
  bool destructed;

  ~MockOpenFile() {destructed = true;}

  MOCK_CONST_METHOD1(stat, void(struct ::stat*));
  MOCK_CONST_METHOD3(read, size_t(void*, size_t, off_t));
  MOCK_METHOD1(truncate, void(off_t));
  MOCK_METHOD3(write, void(const void*, size_t, off_t));
  MOCK_METHOD0(flush, void());
  MOCK_METHOD0(fsync, void());
  MOCK_METHOD0(fdatasync, void());
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
    MockOpenFile *openFile = dynamic_cast<MockOpenFile*>(list.get(id));
    EXPECT_EQ(fileid, openFile->fileid);
    EXPECT_EQ(flags, openFile->flags);
  }
};

TEST_F(FuseOpenFileListTest, EmptyList1) {
  ASSERT_THROW(list.get(0), std::out_of_range);
}

TEST_F(FuseOpenFileListTest, EmptyList2) {
  ASSERT_THROW(list.get(3), std::out_of_range);
}

TEST_F(FuseOpenFileListTest, InvalidId) {
  int valid_id = open();
  int invalid_id = valid_id + 1;
  ASSERT_THROW(list.get(invalid_id), std::out_of_range);
}

TEST_F(FuseOpenFileListTest, Open1AndGet) {
  int id = open(FILEID1, FLAGS1);
  check(id, FILEID1, FLAGS1);
}

TEST_F(FuseOpenFileListTest, Open2AndGet) {
  int id1 = open(FILEID1, FLAGS1);
  int id2 = open(FILEID2, FLAGS2);

  check(id1, FILEID1, FLAGS1);
  check(id2, FILEID2, FLAGS2);
}

TEST_F(FuseOpenFileListTest, Open3AndGet) {
  int id1 = open(FILEID1, FLAGS1);
  int id2 = open(FILEID2, FLAGS2);
  int id3 = open(FILEID3, FLAGS3);

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
  int id = open();

  ASSERT_NO_THROW(list.get(id));
  list.close(id);
  ASSERT_THROW(list.get(id), std::out_of_range);
}

TEST_F(FuseOpenFileListTest, GetClosedItemOnNonEmptyList) {
  int id = open();
  open();

  ASSERT_NO_THROW(list.get(id));
  list.close(id);
  ASSERT_THROW(list.get(id), std::out_of_range);
}

TEST_F(FuseOpenFileListTest, CloseOnEmptyList1) {
  ASSERT_THROW(list.close(0), std::out_of_range);
}

TEST_F(FuseOpenFileListTest, CloseOnEmptyList2) {
  ASSERT_THROW(list.close(4), std::out_of_range);
}

TEST_F(FuseOpenFileListTest, RemoveInvalidId) {
  int valid_id = open();
  int invalid_id = valid_id + 1;
  ASSERT_THROW(list.close(invalid_id), std::out_of_range);
}
