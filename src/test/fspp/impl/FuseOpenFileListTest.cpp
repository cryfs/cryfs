#include "gtest/gtest.h"
#include "gmock/gmock.h"

#include "fspp/impl/FuseOpenFileList.h"

#include <stdexcept>

using std::unique_ptr;
using std::make_unique;

using namespace fspp;

class MockOpenFile: public OpenFile {
public:
  MockOpenFile(int fileid_, int flags_): fileid(fileid_), flags(flags_), destructed(false) {}
  int fileid, flags;
  bool destructed;

  ~MockOpenFile() {destructed = true;}

  MOCK_CONST_METHOD1(stat, void(struct ::stat*));
  MOCK_CONST_METHOD1(truncate, void(off_t));
  MOCK_METHOD3(read, int(void*, size_t, off_t));
  MOCK_METHOD3(write, void(const void*, size_t, off_t));
  MOCK_METHOD0(fsync, void());
  MOCK_METHOD0(fdatasync, void());
};

class MockFile: public File {
public:
  MockFile(int id_): id(id_) {}
  int id;

  unique_ptr<OpenFile> open(int flags) const override {
    return make_unique<MockOpenFile>(id, flags);
  }
  MOCK_CONST_METHOD1(truncate, void(off_t));
  MOCK_METHOD0(unlink, void());
  MOCK_CONST_METHOD1(stat, void(struct ::stat*));
  MOCK_CONST_METHOD1(access, void(int));
  MOCK_METHOD1(rename, void(const boost::filesystem::path &));
  MOCK_METHOD1(utimens, void(const timespec[2]));
};

TEST(FuseOpenFileListTest, EmptyList1) {
  FuseOpenFileList list;
  ASSERT_THROW(list.get(0), std::out_of_range);
}

TEST(FuseOpenFileListTest, EmptyList2) {
  FuseOpenFileList list;
  ASSERT_THROW(list.get(3), std::out_of_range);
}

TEST(FuseOpenFileListTest, InvalidId) {
  FuseOpenFileList list;
  int valid_id = list.open(MockFile(3), 2);
  int invalid_id = valid_id + 1;
  ASSERT_THROW(list.get(invalid_id), std::out_of_range);
}

TEST(FuseOpenFileListTest, Open1AndGet) {
  const int FILEID = 4;
  const int FLAGS = 5;

  FuseOpenFileList list;
  int id = list.open(MockFile(FILEID), FLAGS);

  MockOpenFile *openFile = dynamic_cast<MockOpenFile*>(list.get(id));

  EXPECT_EQ(FILEID, openFile->fileid);
  EXPECT_EQ(FLAGS, openFile->flags);
}

TEST(FuseOpenFileListTest, Open2AndGet) {
  const int FILEID1 = 4;
  const int FLAGS1 = 5;
  const int FILEID2 = 6;
  const int FLAGS2 = 7;

  FuseOpenFileList list;
  int id1 = list.open(MockFile(FILEID1), FLAGS1);
  int id2 = list.open(MockFile(FILEID2), FLAGS2);

  MockOpenFile *openFile1 = dynamic_cast<MockOpenFile*>(list.get(id1));
  MockOpenFile *openFile2 = dynamic_cast<MockOpenFile*>(list.get(id2));

  EXPECT_EQ(FILEID1, openFile1->fileid);
  EXPECT_EQ(FLAGS1, openFile1->flags);
  EXPECT_EQ(FILEID2, openFile2->fileid);
  EXPECT_EQ(FLAGS2, openFile2->flags);
}

TEST(FuseOpenFileListTest, Open3AndGet) {
  const int FILEID1 = 4;
  const int FLAGS1 = 5;
  const int FILEID2 = 6;
  const int FLAGS2 = 7;
  const int FILEID3 = 8;
  const int FLAGS3 = 9;

  FuseOpenFileList list;
  int id1 = list.open(MockFile(FILEID1), FLAGS1);
  int id2 = list.open(MockFile(FILEID2), FLAGS2);
  int id3 = list.open(MockFile(FILEID3), FLAGS3);

  MockOpenFile *openFile1 = dynamic_cast<MockOpenFile*>(list.get(id1));
  MockOpenFile *openFile3 = dynamic_cast<MockOpenFile*>(list.get(id3));
  MockOpenFile *openFile2 = dynamic_cast<MockOpenFile*>(list.get(id2));

  EXPECT_EQ(FILEID1, openFile1->fileid);
  EXPECT_EQ(FLAGS1, openFile1->flags);
  EXPECT_EQ(FILEID2, openFile2->fileid);
  EXPECT_EQ(FLAGS2, openFile2->flags);
  EXPECT_EQ(FILEID3, openFile3->fileid);
  EXPECT_EQ(FLAGS3, openFile3->flags);
}

TEST(FuseOpenFileListTest, DestructOnClose) {
  FuseOpenFileList list;
  int id = list.open(MockFile(3), 4);

  MockOpenFile *openFile = dynamic_cast<MockOpenFile*>(list.get(id));

  EXPECT_FALSE(openFile->destructed);
  list.close(id);
  EXPECT_TRUE(openFile->destructed);
}

TEST(FuseOpenFileListTest, GetClosedItemOnEmptyList) {
  FuseOpenFileList list;
  int id = list.open(MockFile(3), 4);

  ASSERT_NO_THROW(list.get(id));
  list.close(id);
  ASSERT_THROW(list.get(id), std::out_of_range);
}

TEST(FuseOpenFileListTest, GetClosedItemOnNonEmptyList) {
  FuseOpenFileList list;
  int id = list.open(MockFile(3), 4);
  list.open(MockFile(5), 4);

  ASSERT_NO_THROW(list.get(id));
  list.close(id);
  ASSERT_THROW(list.get(id), std::out_of_range);
}

TEST(FuseOpenFileListTest, CloseOnEmptyList1) {
  FuseOpenFileList list;
  ASSERT_THROW(list.close(0), std::out_of_range);
}

TEST(FuseOpenFileListTest, CloseOnEmptyList2) {
  FuseOpenFileList list;
  ASSERT_THROW(list.close(4), std::out_of_range);
}

TEST(FuseOpenFileListTest, RemoveInvalidId) {
  FuseOpenFileList list;
  int valid_id = list.open(MockFile(3), 4);
  int invalid_id = valid_id + 1;
  ASSERT_THROW(list.close(invalid_id), std::out_of_range);
}
