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
  const mode_t MODE_FILE = S_IFREG | S_IRUSR | S_IWGRP | S_IXOTH;
  const mode_t MODE_DIR = S_IFDIR | S_IWUSR | S_IXGRP | S_IROTH;
  const uid_t UID1 = 0;
  const uid_t UID2 = 10;
  const gid_t GID1 = 0;
  const gid_t GID2 = 10;
  const off_t SIZE1 = 0;
  const off_t SIZE2 = 4096;
  const off_t SIZE3 = 1024*1024*1024;
  const nlink_t NLINK1 = 1;
  const nlink_t NLINK2 = 5;

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

  struct stat CallLstatWithMode(mode_t mode) {
    return CallLstatWithModeAndImpl(mode, [](struct stat*){});
  }

  struct stat CallFileLstatWithUid(uid_t uid) {
    return CallFileLstatWithImpl(LstatUidImpl(uid));
  }

  struct stat CallDirLstatWithUid(uid_t uid) {
    return CallDirLstatWithImpl(LstatUidImpl(uid));
  }

  struct stat CallFileLstatWithGid(gid_t gid) {
    return CallFileLstatWithImpl(LstatGidImpl(gid));
  }

  struct stat CallDirLstatWithGid(gid_t gid) {
    return CallDirLstatWithImpl(LstatGidImpl(gid));
  }

  struct stat CallFileLstatWithSize(off_t size) {
    return CallFileLstatWithImpl(LstatSizeImpl(size));
  }

  struct stat CallDirLstatWithSize(off_t size) {
    return CallDirLstatWithImpl(LstatSizeImpl(size));
  }

  struct stat CallFileLstatWithNlink(nlink_t nlink) {
    return CallFileLstatWithImpl(LstatNlinkImpl(nlink));
  }

  struct stat CallDirLstatWithNlink(nlink_t nlink) {
    return CallDirLstatWithImpl(LstatNlinkImpl(nlink));
  }
private:

  static function<void(struct stat*)> LstatUidImpl(uid_t uid) {
    return [uid] (struct stat *stat) {
      stat->st_uid = uid;
    };
  }

  static function<void(struct stat*)> LstatGidImpl(gid_t gid) {
    return [gid] (struct stat *stat) {
      stat->st_gid = gid;
    };
  }

  static function<void(struct stat*)> LstatSizeImpl(off_t size) {
    return [size] (struct stat *stat) {
      stat->st_size = size;
    };
  }

  static function<void(struct stat*)> LstatNlinkImpl(nlink_t nlink) {
    return [nlink] (struct stat *stat) {
      stat->st_nlink = nlink;
    };
  }

  struct stat CallFileLstatWithImpl(function<void(struct stat*)> implementation) {
    return CallLstatWithModeAndImpl(S_IFREG, implementation);
  }

  struct stat CallDirLstatWithImpl(function<void(struct stat*)> implementation) {
    return CallLstatWithModeAndImpl(S_IFDIR, implementation);
  }

  struct stat CallLstatWithModeAndImpl(mode_t mode, function<void(struct stat*)> implementation) {
    return CallLstatWithImpl([mode, implementation] (struct stat *stat) {
      stat->st_mode = mode;
      implementation(stat);
    });
  }

  struct stat CallLstatWithImpl(function<void(struct stat*)> implementation) {
    EXPECT_CALL(fsimpl, lstat(StrEq(FILENAME), _)).WillRepeatedly(Invoke([implementation](const char*, struct ::stat *stat) {
      implementation(stat);
    }));

    struct stat result;
    LstatPath(FILENAME, &result);

    return result;
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

TEST_F(FuseLstatTest, ReturnedFileModeIsCorrect) {
  struct ::stat result = CallLstatWithMode(MODE_FILE);
  EXPECT_EQ(MODE_FILE, result.st_mode);
}

TEST_F(FuseLstatTest, ReturnedDirModeIsCorrect) {
  struct ::stat result = CallLstatWithMode(MODE_DIR);
  EXPECT_EQ(MODE_DIR, result.st_mode);
}

TEST_F(FuseLstatTest, ReturnedFileUidIsCorrect1) {
  struct ::stat result = CallFileLstatWithUid(UID1);
  EXPECT_EQ(UID1, result.st_uid);
}

TEST_F(FuseLstatTest, ReturnedFileUidIsCorrect2) {
  struct ::stat result = CallFileLstatWithUid(UID2);
  EXPECT_EQ(UID2, result.st_uid);
}

TEST_F(FuseLstatTest, ReturnedDirUidIsCorrect1) {
  struct ::stat result = CallDirLstatWithUid(UID1);
  EXPECT_EQ(UID1, result.st_uid);
}

TEST_F(FuseLstatTest, ReturnedDirUidIsCorrect2) {
  struct ::stat result = CallDirLstatWithUid(UID2);
  EXPECT_EQ(UID2, result.st_uid);
}

TEST_F(FuseLstatTest, ReturnedFileGidIsCorrect1) {
  struct ::stat result = CallFileLstatWithUid(GID1);
  EXPECT_EQ(GID1, result.st_gid);
}

TEST_F(FuseLstatTest, ReturnedFileGidIsCorrect2) {
  struct ::stat result = CallFileLstatWithGid(GID2);
  EXPECT_EQ(GID2, result.st_gid);
}

TEST_F(FuseLstatTest, ReturnedDirGidIsCorrect1) {
  struct ::stat result = CallDirLstatWithGid(GID1);
  EXPECT_EQ(GID1, result.st_gid);
}

TEST_F(FuseLstatTest, ReturnedDirGidIsCorrect2) {
  struct ::stat result = CallDirLstatWithGid(GID2);
  EXPECT_EQ(GID2, result.st_gid);
}

TEST_F(FuseLstatTest, ReturnedFileSizeIsCorrect1) {
  struct ::stat result = CallFileLstatWithSize(SIZE1);
  EXPECT_EQ(SIZE1, result.st_size);
}

TEST_F(FuseLstatTest, ReturnedFileSizeIsCorrect2) {
  struct ::stat result = CallFileLstatWithSize(SIZE2);
  EXPECT_EQ(SIZE2, result.st_size);
}

TEST_F(FuseLstatTest, ReturnedFileSizeIsCorrect3) {
  struct ::stat result = CallFileLstatWithSize(SIZE3);
  EXPECT_EQ(SIZE3, result.st_size);
}

TEST_F(FuseLstatTest, ReturnedDirSizeIsCorrect1) {
  struct ::stat result = CallDirLstatWithSize(SIZE1);
  EXPECT_EQ(SIZE1, result.st_size);
}

TEST_F(FuseLstatTest, ReturnedDirSizeIsCorrect2) {
  struct ::stat result = CallDirLstatWithSize(SIZE2);
  EXPECT_EQ(SIZE2, result.st_size);
}

TEST_F(FuseLstatTest, ReturnedDirSizeIsCorrect3) {
  struct ::stat result = CallDirLstatWithSize(SIZE3);
  EXPECT_EQ(SIZE3, result.st_size);
}

TEST_F(FuseLstatTest, ReturnedFileNlinkIsCorrect1) {
  struct ::stat result = CallFileLstatWithNlink(NLINK1);
  EXPECT_EQ(NLINK1, result.st_nlink);
}

TEST_F(FuseLstatTest, ReturnedFileNlinkIsCorrect2) {
  struct ::stat result = CallFileLstatWithNlink(NLINK2);
  EXPECT_EQ(NLINK2, result.st_nlink);
}

TEST_F(FuseLstatTest, ReturnedDirNlinkIsCorrect1) {
  struct ::stat result = CallDirLstatWithNlink(NLINK1);
  EXPECT_EQ(NLINK1, result.st_nlink);
}

TEST_F(FuseLstatTest, ReturnedDirNlinkIsCorrect2) {
  struct ::stat result = CallDirLstatWithNlink(NLINK2);
  EXPECT_EQ(NLINK2, result.st_nlink);
}

//TODO st_atim
//TODO st_mtim
//TODO st_ctim
//TODO Error cases
