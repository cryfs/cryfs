#pragma once
#ifndef MESSMER_FSPP_FSTEST_TESTUTILS_FILETEST_H_
#define MESSMER_FSPP_FSTEST_TESTUTILS_FILETEST_H_

#include "FileSystemTest.h"
#include <cpp-utils/data/Data.h>
#include <cpp-utils/pointer/unique_ref.h>
#include <cpp-utils/system/stat.h>

template<class ConcreteFileSystemTestFixture>
class FileTest: public FileSystemTest<ConcreteFileSystemTestFixture> {
public:
  FileTest(): file_root(), file_nested() {
    // NOLINTBEGIN(cppcoreguidelines-prefer-member-initializer)
    this->LoadDir("/")->createAndOpenFile("myfile", this->MODE_PUBLIC, fspp::uid_t(0), fspp::gid_t(0));
    file_root = this->LoadFile("/myfile");
    file_root_node = this->Load("/myfile");

    this->LoadDir("/")->createDir("mydir", this->MODE_PUBLIC, fspp::uid_t(0), fspp::gid_t(0));
    this->LoadDir("/mydir")->createAndOpenFile("mynestedfile", this->MODE_PUBLIC, fspp::uid_t(0), fspp::gid_t(0));
    file_nested = this->LoadFile("/mydir/mynestedfile");
    file_nested_node = this->Load("/mydir/mynestedfile");

    this->LoadDir("/")->createDir("mydir2", this->MODE_PUBLIC, fspp::uid_t(0), fspp::gid_t(0));
    // NOLINTEND(cppcoreguidelines-prefer-member-initializer)
  }
  std::unique_ptr<fspp::File> file_root;
  std::unique_ptr<fspp::File> file_nested;
  std::unique_ptr<fspp::Node> file_root_node;
  std::unique_ptr<fspp::Node> file_nested_node;

  //TODO IN_STAT still needed after moving it to FsppNodeTest?
  void IN_STAT(fspp::Node *node, std::function<void (const fspp::Node::stat_info&)> callback) {
	  auto st1 = node->stat();
	  callback(st1);
  }

  void EXPECT_SIZE(fspp::num_bytes_t expectedSize, fspp::File *file, fspp::Node *node) {
    IN_STAT(node, [expectedSize] (const fspp::Node::stat_info& st) {
      EXPECT_EQ(expectedSize, st.size);
    });

    EXPECT_NUMBYTES_READABLE(expectedSize, file);
  }

  void EXPECT_NUMBYTES_READABLE(fspp::num_bytes_t expectedSize, fspp::File *file) {
    auto openFile = file->open(fspp::openflags_t::RDONLY());
    cpputils::Data data(expectedSize.value());
    //Try to read one byte more than the expected size
    const fspp::num_bytes_t readBytes = openFile->read(data.data(), expectedSize+fspp::num_bytes_t(1), fspp::num_bytes_t(0));
    //and check that it only read the expected size (but also not less)
    EXPECT_EQ(expectedSize, readBytes);
  }

  void EXPECT_ATIME_EQ(struct timespec expected, const fspp::Node::stat_info& st) {
	  EXPECT_EQ(expected.tv_sec, st.atime.tv_sec);
	  EXPECT_EQ(expected.tv_nsec, st.atime.tv_nsec);
  }

  void EXPECT_MTIME_EQ(struct timespec expected, const fspp::Node::stat_info& st) {
    EXPECT_EQ(expected.tv_sec, st.mtime.tv_sec);
    EXPECT_EQ(expected.tv_nsec, st.mtime.tv_nsec);
  }
};

#endif
