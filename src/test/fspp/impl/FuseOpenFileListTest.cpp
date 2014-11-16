#include "gtest/gtest.h"
#include "gmock/gmock.h"

#include "fspp/impl/FuseOpenFileList.h"

using std::unique_ptr;
using std::make_unique;

using namespace fspp;

class MyOpenFile: public OpenFile {
public:
  MyOpenFile(int fileid_, int flags_) :fileid(fileid_), flags(flags_) {}
  ~MyOpenFile() {}
  int fileid;
  int flags;

  void stat(struct ::stat *) const override {}
  void truncate(off_t) const override {}
  int read(void *, size_t, off_t) override {return 0;}
  void write(const void *, size_t, off_t) override {}
  void fsync() override {}
  void fdatasync() override {}
};

class MyFile: public File {
public:
  MyFile(int id_): id(id_) {}
  int id;

  unique_ptr<OpenFile> open(int flags) const override {
    return make_unique<MyOpenFile>(id, flags);
  }

  void truncate(off_t) const override {}
  void unlink() override {}
  void stat(struct ::stat *) const override {}
  void access(int) const override {}
  void rename(const boost::filesystem::path &) override {}
  void utimens(const timespec[2]) override {}
};

TEST(FuseOpenFileListTest, Open) {
  MyFile file(3);
  FuseOpenFileList list;
  int id = list.open(file, 4);
  EXPECT_EQ(3, dynamic_cast<MyOpenFile*>(list.get(id))->fileid);
  EXPECT_EQ(4, dynamic_cast<MyOpenFile*>(list.get(id))->flags);
}
