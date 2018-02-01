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
    this->CreateDir("/mydir");
    auto operation = [this] () {
        this->LoadDir("/mydir")->createAndOpenFile("childname", S_IFREG, 1000, 1000);
    };
    this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mydir", operation, {this->ExpectDoesntUpdateAccessTimestamp, this->ExpectUpdatesModificationTimestamp, this->ExpectUpdatesMetadataTimestamp});
}

/* TODO Re-enable this test once the root dir handles timestamps correctly
TYPED_TEST_P(FsppDirTest_Timestamps, createAndOpenFile_inRootDir) {
    auto dir = this->LoadDir("/");
    auto operation = [&dir] () {
        dir->createAndOpenFile("childname", S_IFREG, 1000, 1000);
    };
    this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/", operation, {this->ExpectDoesntUpdateAccessTimestamp, this->ExpectUpdatesModificationTimestamp, this->ExpectUpdatesMetadataTimestamp});
}*/

TYPED_TEST_P(FsppDirTest_Timestamps, createAndOpenFile_TimestampsOfCreatedFile) {
    auto dir = this->CreateDir("/mydir");
    timespec lowerBound = now();
    dir->createAndOpenFile("childname", S_IFREG, 1000, 1000);
    timespec upperBound = now();
    cpputils::destruct(std::move(dir));
    auto child = this->Load("/mydir/childname");
    this->EXPECT_ACCESS_TIMESTAMP_BETWEEN        (lowerBound, upperBound, *child);
    this->EXPECT_MODIFICATION_TIMESTAMP_BETWEEN  (lowerBound, upperBound, *child);
    this->EXPECT_METADATACHANGE_TIMESTAMP_BETWEEN(lowerBound, upperBound, *child);
}

TYPED_TEST_P(FsppDirTest_Timestamps, createDir) {
    this->CreateDir("/mydir");
    auto operation = [this] () {
        this->LoadDir("/mydir")->createDir("childname", S_IFDIR, 1000, 1000);
    };
    this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mydir", operation, {this->ExpectDoesntUpdateAccessTimestamp, this->ExpectUpdatesModificationTimestamp, this->ExpectUpdatesMetadataTimestamp});
}

/* TODO Re-enable this test once the root dir handles timestamps correctly
TYPED_TEST_P(FsppDirTest_Timestamps, createDir_inRootDir) {
    auto dir = this->LoadDir("/");
    auto operation = [&dir] () {
        dir->createDir("childname", S_IFDIR, 1000, 1000);
    };
    this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/", operation, {this->ExpectDoesntUpdateAccessTimestamp, this->ExpectUpdatesModificationTimestamp, this->ExpectUpdatesMetadataTimestamp});
}*/

TYPED_TEST_P(FsppDirTest_Timestamps, createDir_TimestampsOfCreatedDir) {
    auto dir = this->CreateDir("/mydir");
    timespec lowerBound = now();
    dir->createDir("childname", S_IFDIR, 1000, 1000);
    timespec upperBound = now();
    cpputils::destruct(std::move(dir));
    auto child = this->Load("/mydir/childname");
    this->EXPECT_ACCESS_TIMESTAMP_BETWEEN        (lowerBound, upperBound, *child);
    this->EXPECT_MODIFICATION_TIMESTAMP_BETWEEN  (lowerBound, upperBound, *child);
    this->EXPECT_METADATACHANGE_TIMESTAMP_BETWEEN(lowerBound, upperBound, *child);
}

TYPED_TEST_P(FsppDirTest_Timestamps, createSymlink) {
    this->CreateDir("/mydir");
    auto operation = [this] () {
        this->LoadDir("/mydir")->createSymlink("childname", "/target", 1000, 1000);
    };
    this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mydir", operation, {this->ExpectDoesntUpdateAccessTimestamp, this->ExpectUpdatesModificationTimestamp, this->ExpectUpdatesMetadataTimestamp});
}

/* TODO Re-enable this test once the root dir handles timestamps correctly
TYPED_TEST_P(FsppDirTest_Timestamps, createSymlink_inRootDir) {
    auto operation = [this] () {
        this->LoadDir("/")->createSymlink("childname", "/target", 1000, 1000);
    };
    this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/", operation, {this->ExpectDoesntUpdateAccessTimestamp, this->ExpectUpdatesModificationTimestamp, this->ExpectUpdatesMetadataTimestamp});
}*/

TYPED_TEST_P(FsppDirTest_Timestamps, createSymlink_TimestampsOfCreatedSymlink) {
    auto dir = this->CreateDir("/mydir");
    timespec lowerBound = now();
    dir->createSymlink("childname", "/target", 1000, 1000);
    timespec upperBound = now();
    cpputils::destruct(std::move(dir));
    auto child = this->Load("/mydir/childname");
    this->EXPECT_ACCESS_TIMESTAMP_BETWEEN        (lowerBound, upperBound, *child);
    this->EXPECT_MODIFICATION_TIMESTAMP_BETWEEN  (lowerBound, upperBound, *child);
    this->EXPECT_METADATACHANGE_TIMESTAMP_BETWEEN(lowerBound, upperBound, *child);
}

TYPED_TEST_P(FsppDirTest_Timestamps, children_empty) {
    this->CreateDir("/mydir");
    this->setModificationTimestampLaterThanAccessTimestamp("/mydir"); // to make sure that even in relatime behavior, the read access below changes the access timestamp
    auto operation = [this] () {
        this->LoadDir("/mydir")->children();
    };
    this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mydir", operation, {this->ExpectUpdatesAccessTimestamp, this->ExpectDoesntUpdateModificationTimestamp, this->ExpectDoesntUpdateMetadataTimestamp});
}

/* TODO Re-enable this test once the root dir handles timestamps correctly
TYPED_TEST_P(FsppDirTest_Timestamps, children_empty_inRootDir) {
    auto operation = [this] () {
        this->LoadDir("/")->children();
    };
    this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/", operation, {this->ExpectUpdatesAccessTimestamp, this->ExpectDoesntUpdateModificationTimestamp, this->ExpectDoesntUpdateMetadataTimestamp});
}*/

TYPED_TEST_P(FsppDirTest_Timestamps, children_nonempty) {
    this->CreateDir("/mydir")->createAndOpenFile("filename", S_IFREG, 1000, 1000);
    auto operation = [this] () {
        this->LoadDir("/mydir")->children();
    };
    this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mydir", operation, {this->ExpectUpdatesAccessTimestamp, this->ExpectDoesntUpdateModificationTimestamp, this->ExpectDoesntUpdateMetadataTimestamp});
}

/* TODO Re-enable this test once the root dir handles timestamps correctly
TYPED_TEST_P(FsppDirTest_Timestamps, children_nonempty_inRootDir) {
    this->LoadDir("/")->createAndOpenFile("filename", S_IFREG, 1000, 1000);
    auto operation = [this] () {
        this->LoadDir("/")->children();
    };
    this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/", operation, {this->ExpectUpdatesAccessTimestamp, this->ExpectDoesntUpdateModificationTimestamp, this->ExpectDoesntUpdateMetadataTimestamp});
}*/

template<class ConcreteFileSystemTestFixture>
class FsppDirTest_Timestamps_Entries: public FsppNodeTest<ConcreteFileSystemTestFixture>, public TimestampTestUtils<ConcreteFileSystemTestFixture> {
public:

    void Test_deleteChild() {
        this->CreateDir("/mydir");
        this->CreateNode("/mydir/childname");
        auto operation = [this]() {
            this->Load("/mydir/childname")->remove();
        };
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mydir", operation, {
            this->ExpectDoesntUpdateAccessTimestamp,
            this->ExpectUpdatesModificationTimestamp,
            this->ExpectUpdatesMetadataTimestamp
        });
    }

    /* TODO Re-enable this test once the root dir handles timestamps correctly
    void Test_deleteChild_inRootDir() {
        this->CreateNode("/childname");
        auto operation = [this] () {
            this->Load("/childname")->remove();
        };
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/", operation, {this->ExpectDoesntUpdateAccessTimestamp, this->ExpectUpdatesModificationTimestamp, this->ExpectUpdatesMetadataTimestamp});
    }*/

    void Test_renameChild() {
        this->CreateDir("/mydir");
        this->CreateNode("/mydir/childname");
        auto operation = [this]() {
            this->Load("/mydir/childname")->rename("/mydir/mychild");
        };
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mydir", operation, {
            this->ExpectDoesntUpdateAccessTimestamp,
            this->ExpectUpdatesModificationTimestamp,
            this->ExpectUpdatesMetadataTimestamp
        });
    }

    /* TODO Re-enable this test once the root dir handles timestamps correctly
    void Test_renameChild_inRootDir() {
        this->CreateNode("/childname");
        auto operation = [this] () {
            this->Load("/childname")->rename("/mydir/mychild");
        };
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/", operation, {this->ExpectDoesntUpdateAccessTimestamp, this->ExpectUpdatesModificationTimestamp, this->ExpectUpdatesMetadataTimestamp});
    }*/

    void Test_moveChildIn() {
        this->CreateDir("/sourcedir");
        this->CreateNode("/sourcedir/childname");
        this->CreateDir("/mydir");
        auto operation = [this]() {
            this->Load("/sourcedir/childname")->rename("/mydir/mychild");
        };
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mydir", operation, {
            this->ExpectDoesntUpdateAccessTimestamp,
            this->ExpectUpdatesModificationTimestamp,
            this->ExpectUpdatesMetadataTimestamp
        });
    }

    /* TODO Re-enable this test once the root dir handles timestamps correctly
    void Test_moveChildIn_inRootDir() {
        this->CreateDir("/sourcedir");
        this->CreateNode("/sourcedir/childname");
        auto operation = [this] () {
            this->Load("/sourcedir/childname")->rename("/mychild");
        };
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/", operation, {this->ExpectDoesntUpdateAccessTimestamp, this->ExpectUpdatesModificationTimestamp, this->ExpectUpdatesMetadataTimestamp});
    }*/

    void Test_moveChildOut() {
        this->CreateDir("/mydir");
        this->CreateNode("/mydir/childname");
        this->CreateDir("/targetdir");
        auto operation = [this]() {
            this->Load("/mydir/childname")->rename("/targetdir/mychild");
        };
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mydir", operation, {
            this->ExpectDoesntUpdateAccessTimestamp,
            this->ExpectUpdatesModificationTimestamp,
            this->ExpectUpdatesMetadataTimestamp
        });
    }

    /* TODO Re-enable this test once the root dir handles timestamps correctly
    void Test_moveChildOut_inRootDir() {
        this->CreateNode("/childname");
        this->CreateDir("/targetdir");
        auto operation = [this] () {
            this->Load("/childname")->rename("/targetdir/mychild");
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
