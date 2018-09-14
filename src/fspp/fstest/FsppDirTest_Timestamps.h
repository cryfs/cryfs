#pragma once
#ifndef MESSMER_FSPP_FSTEST_FSPPDIRTEST_TIMESTAMPS_H_
#define MESSMER_FSPP_FSTEST_FSPPDIRTEST_TIMESTAMPS_H_

#include "testutils/TimestampTestUtils.h"

template<class ConcreteFileSystemTestFixture>
class FsppDirTest_Timestamps: public TimestampTestUtils<ConcreteFileSystemTestFixture> {
public:
};
TYPED_TEST_CASE_P(FsppDirTest_Timestamps);

TYPED_TEST_P(FsppDirTest_Timestamps, createAndOpenFile) {
    auto dir = this->CreateDir("/mydir");
    auto operation = [&dir] () {
        dir->createAndOpenFile("childname", fspp::mode_t().addFileFlag(), fspp::uid_t(1000), fspp::gid_t(1000));
    };
    this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mydir", operation, {this->ExpectDoesntUpdateAccessTimestamp, this->ExpectUpdatesModificationTimestamp, this->ExpectUpdatesMetadataTimestamp});
}

/* TODO Re-enable this test once the root dir handles timestamps correctly
TYPED_TEST_P(FsppDirTest_Timestamps, createAndOpenFile_inRootDir) {
    auto dir = this->LoadDir("/");
    auto operation = [&dir] () {
        dir->createAndOpenFile("childname", fspp::mode_t().addFileFlag(), fspp::uid_t(1000), fspp::gid_t(1000));
    };
    this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/", operation, {this->ExpectDoesntUpdateAccessTimestamp, this->ExpectUpdatesModificationTimestamp, this->ExpectUpdatesMetadataTimestamp});
}*/

TYPED_TEST_P(FsppDirTest_Timestamps, createAndOpenFile_TimestampsOfCreatedFile) {
    auto dir = this->CreateDir("/mydir");
    timespec lowerBound = now();
    dir->createAndOpenFile("childname", fspp::mode_t().addFileFlag(), fspp::uid_t(1000), fspp::gid_t(1000));
    timespec upperBound = now();
    auto child = this->Load("/mydir/childname");
    this->EXPECT_ACCESS_TIMESTAMP_BETWEEN        (lowerBound, upperBound, *child);
    this->EXPECT_MODIFICATION_TIMESTAMP_BETWEEN  (lowerBound, upperBound, *child);
    this->EXPECT_METADATACHANGE_TIMESTAMP_BETWEEN(lowerBound, upperBound, *child);
}

TYPED_TEST_P(FsppDirTest_Timestamps, createDir) {
    auto dir = this->CreateDir("/mydir");
    auto operation = [&dir] () {
        dir->createDir("childname", fspp::mode_t().addDirFlag(), fspp::uid_t(1000), fspp::gid_t(1000));
    };
    this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mydir", operation, {this->ExpectDoesntUpdateAccessTimestamp, this->ExpectUpdatesModificationTimestamp, this->ExpectUpdatesMetadataTimestamp});
}

/* TODO Re-enable this test once the root dir handles timestamps correctly
TYPED_TEST_P(FsppDirTest_Timestamps, createDir_inRootDir) {
    auto dir = this->LoadDir("/");
    auto operation = [&dir] () {
        dir->createDir("childname", fspp::mode_t().addDirFlag(), fspp::uid_t(1000), fspp::gid_t(1000));
    };
    this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/", operation, {this->ExpectDoesntUpdateAccessTimestamp, this->ExpectUpdatesModificationTimestamp, this->ExpectUpdatesMetadataTimestamp});
}*/

TYPED_TEST_P(FsppDirTest_Timestamps, createDir_TimestampsOfCreatedDir) {
    auto dir = this->CreateDir("/mydir");
    timespec lowerBound = now();
    dir->createDir("childname", fspp::mode_t().addDirFlag(), fspp::uid_t(1000), fspp::gid_t(1000));
    timespec upperBound = now();
    auto child = this->Load("/mydir/childname");
    this->EXPECT_ACCESS_TIMESTAMP_BETWEEN        (lowerBound, upperBound, *child);
    this->EXPECT_MODIFICATION_TIMESTAMP_BETWEEN  (lowerBound, upperBound, *child);
    this->EXPECT_METADATACHANGE_TIMESTAMP_BETWEEN(lowerBound, upperBound, *child);
}

TYPED_TEST_P(FsppDirTest_Timestamps, createSymlink) {
    auto dir = this->CreateDir("/mydir");
    auto operation = [&dir] () {
        dir->createSymlink("childname", "/target", fspp::uid_t(1000), fspp::gid_t(1000));
    };
    this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mydir", operation, {this->ExpectDoesntUpdateAccessTimestamp, this->ExpectUpdatesModificationTimestamp, this->ExpectUpdatesMetadataTimestamp});
}

/* TODO Re-enable this test once the root dir handles timestamps correctly
TYPED_TEST_P(FsppDirTest_Timestamps, createSymlink_inRootDir) {
    auto dir = this->LoadDir("/");
    auto operation = [&dir] () {
        dir->createSymlink("childname", "/target", fspp::uid_t(1000), fspp::gid_t(1000));
    };
    this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/", operation, {this->ExpectDoesntUpdateAccessTimestamp, this->ExpectUpdatesModificationTimestamp, this->ExpectUpdatesMetadataTimestamp});
}*/

TYPED_TEST_P(FsppDirTest_Timestamps, createSymlink_TimestampsOfCreatedSymlink) {
    auto dir = this->CreateDir("/mydir");
    timespec lowerBound = now();
    dir->createSymlink("childname", "/target", fspp::uid_t(1000), fspp::gid_t(1000));
    timespec upperBound = now();
    auto child = this->Load("/mydir/childname");
    this->EXPECT_ACCESS_TIMESTAMP_BETWEEN        (lowerBound, upperBound, *child);
    this->EXPECT_MODIFICATION_TIMESTAMP_BETWEEN  (lowerBound, upperBound, *child);
    this->EXPECT_METADATACHANGE_TIMESTAMP_BETWEEN(lowerBound, upperBound, *child);
}

TYPED_TEST_P(FsppDirTest_Timestamps, children_empty) {
    auto dir = this->CreateDir("/mydir");
    this->setModificationTimestampLaterThanAccessTimestamp("/mydir"); // to make sure that even in relatime behavior, the read access below changes the access timestamp
    auto operation = [&dir] () {
        dir->children();
    };
    this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mydir", operation, {this->ExpectUpdatesAccessTimestamp, this->ExpectDoesntUpdateModificationTimestamp, this->ExpectDoesntUpdateMetadataTimestamp});
}

/* TODO Re-enable this test once the root dir handles timestamps correctly
TYPED_TEST_P(FsppDirTest_Timestamps, children_empty_inRootDir) {
    auto dir = this->LoadDir("/");
    auto operation = [&dir] () {
        dir->children();
    };
    this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/", operation, {this->ExpectUpdatesAccessTimestamp, this->ExpectDoesntUpdateModificationTimestamp, this->ExpectDoesntUpdateMetadataTimestamp});
}*/

TYPED_TEST_P(FsppDirTest_Timestamps, children_nonempty) {
    auto dir = this->CreateDir("/mydir");
    dir->createAndOpenFile("filename", fspp::mode_t().addFileFlag(), fspp::uid_t(1000), fspp::gid_t(1000));
    auto operation = [&dir] () {
        dir->children();
    };
    this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mydir", operation, {this->ExpectUpdatesAccessTimestamp, this->ExpectDoesntUpdateModificationTimestamp, this->ExpectDoesntUpdateMetadataTimestamp});
}

/* TODO Re-enable this test once the root dir handles timestamps correctly
TYPED_TEST_P(FsppDirTest_Timestamps, children_nonempty_inRootDir) {
    auto dir = this->LoadDir("/");
    dir->createAndOpenFile("filename", fspp::mode_t().addFileFlag(), fspp::uid_t(1000), fspp::gid_t(1000));
    auto operation = [&dir] () {
        dir->children();
    };
    this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/", operation, {this->ExpectUpdatesAccessTimestamp, this->ExpectDoesntUpdateModificationTimestamp, this->ExpectDoesntUpdateMetadataTimestamp});
}*/

template<class ConcreteFileSystemTestFixture>
class FsppDirTest_Timestamps_Entries: public FsppNodeTest<ConcreteFileSystemTestFixture>, public TimestampTestUtils<ConcreteFileSystemTestFixture> {
public:

    void Test_deleteChild() {
        auto dir = this->CreateDir("/mydir");
        auto child = this->CreateNode("/mydir/childname");
        auto operation = [&child]() {
            child->remove();
        };
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mydir", operation, {
            this->ExpectDoesntUpdateAccessTimestamp,
            this->ExpectUpdatesModificationTimestamp,
            this->ExpectUpdatesMetadataTimestamp
        });
    }

    /* TODO Re-enable this test once the root dir handles timestamps correctly
    void Test_deleteChild_inRootDir() {
        auto dir = this->LoadDir("/");
        auto child = this->CreateNode("/childname");
        auto operation = [&child] () {
            child->remove();
        };
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/", operation, {this->ExpectDoesntUpdateAccessTimestamp, this->ExpectUpdatesModificationTimestamp, this->ExpectUpdatesMetadataTimestamp});
    }*/

    void Test_renameChild() {
        auto dir = this->CreateDir("/mydir");
        auto child = this->CreateNode("/mydir/childname");
        auto operation = [&child]() {
            child->rename("/mydir/mychild");
        };
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mydir", operation, {
            this->ExpectDoesntUpdateAccessTimestamp,
            this->ExpectUpdatesModificationTimestamp,
            this->ExpectUpdatesMetadataTimestamp
        });
    }

    /* TODO Re-enable this test once the root dir handles timestamps correctly
    void Test_renameChild_inRootDir() {
        auto dir = this->LoadDir("/");
        auto child = this->CreateNode("/childname");
        auto operation = [&child] () {
            child->rename("/mydir/mychild");
        };
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/", operation, {this->ExpectDoesntUpdateAccessTimestamp, this->ExpectUpdatesModificationTimestamp, this->ExpectUpdatesMetadataTimestamp});
    }*/

    void Test_moveChildIn() {
        auto sourceDir = this->CreateDir("/sourcedir");
        auto child = this->CreateNode("/sourcedir/childname");
        auto dir = this->CreateDir("/mydir");
        auto operation = [&child]() {
            child->rename("/mydir/mychild");
        };
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mydir", operation, {
            this->ExpectDoesntUpdateAccessTimestamp,
            this->ExpectUpdatesModificationTimestamp,
            this->ExpectUpdatesMetadataTimestamp
        });
    }

    /* TODO Re-enable this test once the root dir handles timestamps correctly
    void Test_moveChildIn_inRootDir() {
        auto sourceDir = this->CreateDir("/sourcedir");
        auto child = this->CreateNode("/sourcedir/childname");
        auto dir = this->LoadDir("/");
        auto operation = [&child] () {
            child->rename("/mychild");
        };
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/", operation, {this->ExpectDoesntUpdateAccessTimestamp, this->ExpectUpdatesModificationTimestamp, this->ExpectUpdatesMetadataTimestamp});
    }*/

    void Test_moveChildOut() {
        auto dir = this->CreateDir("/mydir");
        auto child = this->CreateNode("/mydir/childname");
        this->CreateDir("/targetdir");
        auto operation = [&child]() {
            child->rename("/targetdir/mychild");
        };
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mydir", operation, {
            this->ExpectDoesntUpdateAccessTimestamp,
            this->ExpectUpdatesModificationTimestamp,
            this->ExpectUpdatesMetadataTimestamp
        });
    }

    /* TODO Re-enable this test once the root dir handles timestamps correctly
    void Test_moveChildOut_inRootDir() {
        auto dir = this->LoadDir("/");
        auto child = this->CreateNode("/childname");
        this->CreateDir("/targetdir");
        auto operation = [&child] () {
            child->rename("/targetdir/mychild");
        };
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/", operation, {this->ExpectDoesntUpdateAccessTimestamp, this->ExpectUpdatesModificationTimestamp, this->ExpectUpdatesMetadataTimestamp});
    }*/
};

REGISTER_TYPED_TEST_CASE_P(FsppDirTest_Timestamps,
   createAndOpenFile,
   createAndOpenFile_TimestampsOfCreatedFile,
   createDir,
   createDir_TimestampsOfCreatedDir,
   createSymlink,
   createSymlink_TimestampsOfCreatedSymlink,
   children_empty,
   children_nonempty
);

REGISTER_NODE_TEST_CASE(FsppDirTest_Timestamps_Entries,
   deleteChild,
   renameChild,
   moveChildIn,
   moveChildOut
);

#endif
