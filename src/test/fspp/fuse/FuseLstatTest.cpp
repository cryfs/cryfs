#include "gtest/gtest.h"
#include "gmock/gmock.h"

#include <functional>

#include "test/testutils/FuseTest.h"

using namespace fspp;
using namespace fspp::fuse;

using ::testing::_;
using ::testing::StrEq;
using ::testing::Action;
using ::testing::Invoke;

using std::vector;
using std::string;
using std::function;

class FuseLstatTest: public FuseTest {
public:
  const char *FILENAME = "/myfile";

  void LstatPath(const string &path) {
    struct stat dummy;
    LstatPath(path, &dummy);
  }

  void LstatPath(const string &path, struct stat *result) {
    auto fs = TestFS();

    auto realpath = fs->mountDir() / path;
    int retval = ::lstat(realpath.c_str(), result);
    EXPECT_EQ(0, retval) << "lstat syscall failed. errno: " << errno;
  }

protected:
  struct stat CallFileLstatWithImpl(function<void(struct stat*)> implementation) {
    return CallLstatWithModeAndImpl(S_IFREG, implementation);
  }

  struct stat CallDirLstatWithImpl(function<void(struct stat*)> implementation) {
    return CallLstatWithModeAndImpl(S_IFDIR, implementation);
  }

  struct stat CallLstatWithImpl(function<void(struct stat*)> implementation) {
    EXPECT_CALL(fsimpl, lstat(StrEq(FILENAME), _)).WillRepeatedly(Invoke([implementation](const char*, struct ::stat *stat) {
      implementation(stat);
    }));

    struct stat result;
    LstatPath(FILENAME, &result);

    return result;
  }

private:

  struct stat CallLstatWithModeAndImpl(mode_t mode, function<void(struct stat*)> implementation) {
    return CallLstatWithImpl([mode, implementation] (struct stat *stat) {
      stat->st_mode = mode;
      implementation(stat);
    });
  }
};

template<typename Property>
class FuseLstatReturnPropertyTest: public FuseLstatTest {
public:
  struct stat CallFileLstatWithValue(Property value) {
    return CallFileLstatWithImpl(SetPropertyImpl(value));
  }
  struct stat CallDirLstatWithValue(Property value) {
    return CallDirLstatWithImpl(SetPropertyImpl(value));
  }
private:
  function<void(struct stat*)> SetPropertyImpl(Property value) {
    return [this, value] (struct stat *stat) {
      set(stat, value);
    };
  }
  virtual void set(struct stat *stat, Property value) = 0;
};

class FuseLstatReturnPropertyModeTest: public FuseLstatTest {
public:
  const mode_t MODE1 = S_IFREG | S_IRUSR | S_IWGRP | S_IXOTH;
  const mode_t MODE2 = S_IFDIR | S_IWUSR | S_IXGRP | S_IROTH;

  struct stat CallLstatWithValue(mode_t mode) {
    return CallLstatWithImpl([mode] (struct stat *stat) {
      stat->st_mode = mode;
    });
  }
};

class FuseLstatReturnPropertyUidTest: public FuseLstatReturnPropertyTest<uid_t> {
public:
  const uid_t UID1 = 0;
  const uid_t UID2 = 10;
private:
  void set(struct stat *stat, uid_t value) override {
    stat->st_uid = value;
  }
};

class FuseLstatReturnPropertyGidTest: public FuseLstatReturnPropertyTest<gid_t> {
public:
  const gid_t GID1 = 0;
  const gid_t GID2 = 10;
private:
  void set(struct stat *stat, gid_t value) override {
    stat->st_gid = value;
  }
};

class FuseLstatReturnPropertySizeTest: public FuseLstatReturnPropertyTest<off_t> {
public:
  const off_t SIZE1 = 0;
  const off_t SIZE2 = 4096;
  const off_t SIZE3 = 1024*1024*1024;
private:
  void set(struct stat *stat, off_t value) override {
    stat->st_size = value;
  }
};

class FuseLstatReturnPropertyNlinkTest: public FuseLstatReturnPropertyTest<nlink_t> {
public:
  const nlink_t NLINK1 = 1;
  const nlink_t NLINK2 = 5;
private:
  void set(struct stat *stat, nlink_t value) override {
    stat->st_nlink = value;
  }
};


TEST_F(FuseLstatTest, PathParameterIsCorrectRoot) {
  EXPECT_CALL(fsimpl, lstat(StrEq("/"), _)).Times(1).WillOnce(ReturnIsDir);
  LstatPath("/");
}

TEST_F(FuseLstatTest, PathParameterIsCorrectSimpleFile) {
  EXPECT_CALL(fsimpl, lstat(StrEq("/myfile"), _)).Times(1).WillOnce(ReturnIsFile);
  LstatPath("/myfile");
}

TEST_F(FuseLstatTest, PathParameterIsCorrectSimpleDir) {
  EXPECT_CALL(fsimpl, lstat(StrEq("/mydir"), _)).Times(1).WillOnce(ReturnIsDir);
  LstatPath("/mydir/");
}

TEST_F(FuseLstatTest, PathParameterIsCorrectNestedFile) {
  ReturnIsDirOnLstat("/mydir");
  ReturnIsDirOnLstat("/mydir/mydir2");
  EXPECT_CALL(fsimpl, lstat(StrEq("/mydir/mydir2/myfile"), _)).Times(1).WillOnce(ReturnIsFile);
  LstatPath("/mydir/mydir2/myfile");
}

TEST_F(FuseLstatTest, PathParameterIsCorrectNestedDir) {
  ReturnIsDirOnLstat("/mydir");
  ReturnIsDirOnLstat("/mydir/mydir2");
  EXPECT_CALL(fsimpl, lstat(StrEq("/mydir/mydir2/mydir3"), _)).Times(1).WillOnce(ReturnIsDir);
  LstatPath("/mydir/mydir2/mydir3/");
}

TEST_F(FuseLstatReturnPropertyModeTest, ReturnedModeIsCorrect1) {
  struct ::stat result = CallLstatWithValue(MODE1);
  EXPECT_EQ(MODE1, result.st_mode);
}

TEST_F(FuseLstatReturnPropertyModeTest, ReturnedModeIsCorrect2) {
  struct ::stat result = CallLstatWithValue(MODE2);
  EXPECT_EQ(MODE2, result.st_mode);
}

TEST_F(FuseLstatReturnPropertyUidTest, ReturnedFileUidIsCorrect1) {
  struct ::stat result = CallFileLstatWithValue(UID1);
  EXPECT_EQ(UID1, result.st_uid);
}

TEST_F(FuseLstatReturnPropertyUidTest, ReturnedFileUidIsCorrect2) {
  struct ::stat result = CallFileLstatWithValue(UID2);
  EXPECT_EQ(UID2, result.st_uid);
}

TEST_F(FuseLstatReturnPropertyUidTest, ReturnedDirUidIsCorrect1) {
  struct ::stat result = CallDirLstatWithValue(UID1);
  EXPECT_EQ(UID1, result.st_uid);
}

TEST_F(FuseLstatReturnPropertyUidTest, ReturnedDirUidIsCorrect2) {
  struct ::stat result = CallDirLstatWithValue(UID2);
  EXPECT_EQ(UID2, result.st_uid);
}

TEST_F(FuseLstatReturnPropertyGidTest, ReturnedFileGidIsCorrect1) {
  struct ::stat result = CallFileLstatWithValue(GID1);
  EXPECT_EQ(GID1, result.st_gid);
}

TEST_F(FuseLstatReturnPropertyGidTest, ReturnedFileGidIsCorrect2) {
  struct ::stat result = CallFileLstatWithValue(GID2);
  EXPECT_EQ(GID2, result.st_gid);
}

TEST_F(FuseLstatReturnPropertyGidTest, ReturnedDirGidIsCorrect1) {
  struct ::stat result = CallDirLstatWithValue(GID1);
  EXPECT_EQ(GID1, result.st_gid);
}

TEST_F(FuseLstatReturnPropertyGidTest, ReturnedDirGidIsCorrect2) {
  struct ::stat result = CallDirLstatWithValue(GID2);
  EXPECT_EQ(GID2, result.st_gid);
}

TEST_F(FuseLstatReturnPropertySizeTest, ReturnedFileSizeIsCorrect1) {
  struct ::stat result = CallDirLstatWithValue(SIZE1);
  EXPECT_EQ(SIZE1, result.st_size);
}

TEST_F(FuseLstatReturnPropertySizeTest, ReturnedFileSizeIsCorrect2) {
  struct ::stat result = CallDirLstatWithValue(SIZE2);
  EXPECT_EQ(SIZE2, result.st_size);
}

TEST_F(FuseLstatReturnPropertySizeTest, ReturnedFileSizeIsCorrect3) {
  struct ::stat result = CallDirLstatWithValue(SIZE3);
  EXPECT_EQ(SIZE3, result.st_size);
}

TEST_F(FuseLstatReturnPropertySizeTest, ReturnedDirSizeIsCorrect1) {
  struct ::stat result = CallDirLstatWithValue(SIZE1);
  EXPECT_EQ(SIZE1, result.st_size);
}

TEST_F(FuseLstatReturnPropertySizeTest, ReturnedDirSizeIsCorrect2) {
  struct ::stat result = CallDirLstatWithValue(SIZE2);
  EXPECT_EQ(SIZE2, result.st_size);
}

TEST_F(FuseLstatReturnPropertySizeTest, ReturnedDirSizeIsCorrect3) {
  struct ::stat result = CallDirLstatWithValue(SIZE3);
  EXPECT_EQ(SIZE3, result.st_size);
}

TEST_F(FuseLstatReturnPropertyNlinkTest, ReturnedFileNlinkIsCorrect1) {
  struct ::stat result = CallDirLstatWithValue(NLINK1);
  EXPECT_EQ(NLINK1, result.st_nlink);
}

TEST_F(FuseLstatReturnPropertyNlinkTest, ReturnedFileNlinkIsCorrect2) {
  struct ::stat result = CallDirLstatWithValue(NLINK2);
  EXPECT_EQ(NLINK2, result.st_nlink);
}

TEST_F(FuseLstatReturnPropertyNlinkTest, ReturnedDirNlinkIsCorrect1) {
  struct ::stat result = CallDirLstatWithValue(NLINK1);
  EXPECT_EQ(NLINK1, result.st_nlink);
}

TEST_F(FuseLstatReturnPropertyNlinkTest, ReturnedDirNlinkIsCorrect2) {
  struct ::stat result = CallDirLstatWithValue(NLINK2);
  EXPECT_EQ(NLINK2, result.st_nlink);
}

//TODO st_atim
//TODO st_mtim
//TODO st_ctim
//TODO Error cases
