#pragma once
#ifndef MESSMER_FSPP_FSTEST_FSPPDIRTEST_TIMESTAMPS_H_
#define MESSMER_FSPP_FSTEST_FSPPDIRTEST_TIMESTAMPS_H_

#include "testutils/TimestampTestUtils.h"

template<class ConcreteFileSystemTestFixture>
class FsppDirTest_Timestamps: public TimestampTestUtils<ConcreteFileSystemTestFixture> {
public:
};
TYPED_TEST_SUITE_P(FsppDirTest_Timestamps);

TYPED_TEST_P(FsppDirTest_Timestamps, createAndOpenFile) {
    auto operation = [this] {
        auto dir = this->CreateDir("/mydir");
        return [dir = std::move(dir)] {
            dir->createAndOpenFile("childname", fspp::mode_t().addFileFlag(), fspp::uid_t(1000), fspp::gid_t(1000));
        };
    };
    this->testBuilder().withAnyAtimeConfig([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mydir", operation(), {this->ExpectDoesntUpdateAccessTimestamp, this->ExpectUpdatesModificationTimestamp, this->ExpectUpdatesMetadataTimestamp});
    });
}

/* TODO Re-enable this test once the root dir handles timestamps correctly
TYPED_TEST_P(FsppDirTest_Timestamps, createAndOpenFile_inRootDir) {
     auto operation = [this] {
        auto dir = this->LoadDir("/");
        return [dir = std::move(dir)] {
            dir->createAndOpenFile("childname", fspp::mode_t().addFileFlag(), fspp::uid_t(1000), fspp::gid_t(1000));
        };
    };
    this->testBuilder().withAnyAtimeConfig([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/", operation(), {this->ExpectDoesntUpdateAccessTimestamp, this->ExpectUpdatesModificationTimestamp, this->ExpectUpdatesMetadataTimestamp});
    });
}*/

TYPED_TEST_P(FsppDirTest_Timestamps, createAndOpenFile_TimestampsOfCreatedFile) {
    this->testBuilder().withAnyAtimeConfig([&] {
        auto dir = this->CreateDir("/mydir");
        timespec lowerBound = cpputils::time::now();
        dir->createAndOpenFile("childname", fspp::mode_t().addFileFlag(), fspp::uid_t(1000), fspp::gid_t(1000));
        timespec upperBound = cpputils::time::now();
        auto child = this->Load("/mydir/childname");
        this->EXPECT_ACCESS_TIMESTAMP_BETWEEN        (lowerBound, upperBound, *child);
        this->EXPECT_MODIFICATION_TIMESTAMP_BETWEEN  (lowerBound, upperBound, *child);
        this->EXPECT_METADATACHANGE_TIMESTAMP_BETWEEN(lowerBound, upperBound, *child);
    });
}

TYPED_TEST_P(FsppDirTest_Timestamps, createDir) {
    auto operation = [this] {
        auto dir = this->CreateDir("/mydir");
        return [dir = std::move(dir)] {
            dir->createDir("childname", fspp::mode_t().addDirFlag(), fspp::uid_t(1000), fspp::gid_t(1000));
        };
    };
    this->testBuilder().withAnyAtimeConfig([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mydir", operation(), {this->ExpectDoesntUpdateAccessTimestamp, this->ExpectUpdatesModificationTimestamp, this->ExpectUpdatesMetadataTimestamp});
    });
}

/* TODO Re-enable this test once the root dir handles timestamps correctly
TYPED_TEST_P(FsppDirTest_Timestamps, createDir_inRootDir) {
     auto operation = [this] {
        auto dir = this->LoadDir("/");
        return [dir = std::move(dir)] {
            dir->createDir("childname", fspp::mode_t().addDirFlag(), fspp::uid_t(1000), fspp::gid_t(1000));
        };
    };
    this->testBuilder().withAnyAtimeConfig([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/", operation(), {this->ExpectDoesntUpdateAccessTimestamp, this->ExpectUpdatesModificationTimestamp, this->ExpectUpdatesMetadataTimestamp});
    });
}
*/

TYPED_TEST_P(FsppDirTest_Timestamps, createDir_TimestampsOfCreatedDir) {
    this->testBuilder().withAnyAtimeConfig([&] {
        auto dir = this->CreateDir("/mydir");
        timespec lowerBound = cpputils::time::now();
        dir->createDir("childname", fspp::mode_t().addDirFlag(), fspp::uid_t(1000), fspp::gid_t(1000));
        timespec upperBound = cpputils::time::now();
        auto child = this->Load("/mydir/childname");
        this->EXPECT_ACCESS_TIMESTAMP_BETWEEN        (lowerBound, upperBound, *child);
        this->EXPECT_MODIFICATION_TIMESTAMP_BETWEEN  (lowerBound, upperBound, *child);
        this->EXPECT_METADATACHANGE_TIMESTAMP_BETWEEN(lowerBound, upperBound, *child);
    });
}

TYPED_TEST_P(FsppDirTest_Timestamps, createSymlink) {
    auto operation = [this] {
        auto dir = this->CreateDir("/mydir");
        return [dir = std::move(dir)] {
            dir->createSymlink("childname", "/target", fspp::uid_t(1000), fspp::gid_t(1000));
        };
    };
    this->testBuilder().withAnyAtimeConfig([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mydir", operation(), {this->ExpectDoesntUpdateAccessTimestamp, this->ExpectUpdatesModificationTimestamp, this->ExpectUpdatesMetadataTimestamp});
    });
}

/* TODO Re-enable this test once the root dir handles timestamps correctly
TYPED_TEST_P(FsppDirTest_Timestamps, createSymlink_inRootDir) {
     auto operation = [this] {
        auto dir = this->LoadDir("/");
        return [dir = std::move(dir)] {
            dir->createSymlink("childname", "/target", fspp::uid_t(1000), fspp::gid_t(1000));
        };
    };
    this->testBuilder().withAnyAtimeConfig([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/", operation(), {this->ExpectDoesntUpdateAccessTimestamp, this->ExpectUpdatesModificationTimestamp, this->ExpectUpdatesMetadataTimestamp});
    });
}
*/

TYPED_TEST_P(FsppDirTest_Timestamps, createSymlink_TimestampsOfCreatedSymlink) {
    this->testBuilder().withAnyAtimeConfig([&] {
        auto dir = this->CreateDir("/mydir");
        timespec lowerBound = cpputils::time::now();
        dir->createSymlink("childname", "/target", fspp::uid_t(1000), fspp::gid_t(1000));
        timespec upperBound = cpputils::time::now();
        auto child = this->Load("/mydir/childname");
        this->EXPECT_ACCESS_TIMESTAMP_BETWEEN        (lowerBound, upperBound, *child);
        this->EXPECT_MODIFICATION_TIMESTAMP_BETWEEN  (lowerBound, upperBound, *child);
        this->EXPECT_METADATACHANGE_TIMESTAMP_BETWEEN(lowerBound, upperBound, *child);
    });
}

TYPED_TEST_P(FsppDirTest_Timestamps, givenAtimeOlderThanMtime_children_empty) {
    auto operation = [this] {
        auto dir = this->CreateDir("/mydir");
        this->setAtimeOlderThanMtime("/mydir");
        return [dir = std::move(dir)] {
            dir->children();
        };
    };
    this->testBuilder().withNoatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mydir", operation(), {this->ExpectDoesntUpdateAnyTimestamps});
    }).withStrictatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mydir", operation(), {this->ExpectUpdatesAccessTimestamp, this->ExpectDoesntUpdateModificationTimestamp, this->ExpectDoesntUpdateMetadataTimestamp});
    }).withRelatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mydir", operation(), {this->ExpectUpdatesAccessTimestamp, this->ExpectDoesntUpdateModificationTimestamp, this->ExpectDoesntUpdateMetadataTimestamp});
    }).withNodiratimeRelatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mydir", operation(), {this->ExpectDoesntUpdateAnyTimestamps});
    }).withNodiratimeStrictatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mydir", operation(), {this->ExpectDoesntUpdateAnyTimestamps});
    });
}

TYPED_TEST_P(FsppDirTest_Timestamps, givenAtimeNewerThanMtime_children_empty) {
    auto operation = [this] {
        auto dir = this->CreateDir("/mydir");
        this->setAtimeNewerThanMtime("/mydir");
        return [dir = std::move(dir)] {
            dir->children();
        };
    };
    this->testBuilder().withNoatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mydir", operation(), {this->ExpectDoesntUpdateAnyTimestamps});
    }).withStrictatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mydir", operation(), {this->ExpectUpdatesAccessTimestamp, this->ExpectDoesntUpdateModificationTimestamp, this->ExpectDoesntUpdateMetadataTimestamp});
    }).withRelatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mydir", operation(), {this->ExpectDoesntUpdateAnyTimestamps});
    }).withNodiratimeRelatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mydir", operation(), {this->ExpectDoesntUpdateAnyTimestamps});
    }).withNodiratimeStrictatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mydir", operation(), {this->ExpectDoesntUpdateAnyTimestamps});
    });
}

TYPED_TEST_P(FsppDirTest_Timestamps, givenAtimeNewerThanMtimeButBeforeYesterday_children_empty) {
    auto operation = [this] {
        auto dir = this->CreateDir("/mydir");
        this->setAtimeNewerThanMtimeButBeforeYesterday("/mydir");
        return [dir = std::move(dir)] {
            dir->children();
        };
    };
    this->testBuilder().withNoatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mydir", operation(), {this->ExpectDoesntUpdateAnyTimestamps});
    }).withStrictatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mydir", operation(), {this->ExpectUpdatesAccessTimestamp, this->ExpectDoesntUpdateModificationTimestamp, this->ExpectDoesntUpdateMetadataTimestamp});
    }).withRelatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mydir", operation(), {this->ExpectUpdatesAccessTimestamp, this->ExpectDoesntUpdateModificationTimestamp, this->ExpectDoesntUpdateMetadataTimestamp});
    }).withNodiratimeRelatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mydir", operation(), {this->ExpectDoesntUpdateAnyTimestamps});
    }).withNodiratimeStrictatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mydir", operation(), {this->ExpectDoesntUpdateAnyTimestamps});
    });
}

/* TODO Re-enable this test once the root dir handles timestamps correctly
TYPED_TEST_P(FsppDirTest_Timestamps, givenAtimeOlderThanMtime_children_empty_inRootDir) {
    auto operation = [this] {
        auto dir = this->LoadDir("/");
        this->setAtimeOlderThanMtime("/");
        return [dir = std::move(dir)] {
            dir->children();
        };
    };
    this->testBuilder().withNoatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/", operation(), {this->ExpectDoesntUpdateAnyTimestamps});
    }).withStrictatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/", operation(), {this->ExpectUpdatesAccessTimestamp, this->ExpectDoesntUpdateModificationTimestamp, this->ExpectDoesntUpdateMetadataTimestamp});
    }).withRelatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/", operation(), {this->ExpectUpdatesAccessTimestamp, this->ExpectDoesntUpdateModificationTimestamp, this->ExpectDoesntUpdateMetadataTimestamp});
    }).withNodiratimeRelatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/", operation(), {this->ExpectDoesntUpdateAnyTimestamps});
    }).withNodiratimeStrictatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/", operation(), {this->ExpectDoesntUpdateAnyTimestamps});
    });
}

TYPED_TEST_P(FsppDirTest_Timestamps, givenAtimeNewerThanMtime_children_empty_inRootDir) {
    auto operation = [this] {
        auto dir = this->LoadDir("/");
        this->setAtimeNewerThanMtime("/");
        return [dir = std::move(dir)] {
            dir->children();
        };
    };
    this->testBuilder().withNoatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/", operation(), {this->ExpectDoesntUpdateAnyTimestamps});
    }).withStrictatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/", operation(), {this->ExpectUpdatesAccessTimestamp, this->ExpectDoesntUpdateModificationTimestamp, this->ExpectDoesntUpdateMetadataTimestamp});
    }).withRelatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/", operation(), {this->ExpectDoesntUpdateAnyTimestamps});
    }).withNodiratimeRelatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/", operation(), {this->ExpectDoesntUpdateAnyTimestamps});
    }).withNodiratimeStrictatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/", operation(), {this->ExpectDoesntUpdateAnyTimestamps});
    });
}

TYPED_TEST_P(FsppDirTest_Timestamps, givenAtimeNewerThanMtimeButBeforeYesterday_children_empty_inRootDir) {
    auto operation = [this] {
        auto dir = this->LoadDir("/");
        this->setAtimeNewerThanMtimeButBeforeYesterday("/");
        return [dir = std::move(dir)] {
            dir->children();
        };
    };
    this->testBuilder().withNoatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/", operation(), {this->ExpectDoesntUpdateAnyTimestamps});
    }).withStrictatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/", operation(), {this->ExpectUpdatesAccessTimestamp, this->ExpectDoesntUpdateModificationTimestamp, this->ExpectDoesntUpdateMetadataTimestamp});
    }).withRelatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/", operation(), {this->ExpectUpdatesAccessTimestamp, this->ExpectDoesntUpdateModificationTimestamp, this->ExpectDoesntUpdateMetadataTimestamp});
    }).withNodiratimeRelatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/", operation(), {this->ExpectDoesntUpdateAnyTimestamps});
    }).withNodiratimeStrictatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/", operation(), {this->ExpectDoesntUpdateAnyTimestamps});
    });
}
*/

TYPED_TEST_P(FsppDirTest_Timestamps, givenAtimeOlderThanMtime_children_nonempty) {
    auto operation = [this] {
        auto dir = this->CreateDir("/mydir");
        dir->createAndOpenFile("filename", fspp::mode_t().addFileFlag(), fspp::uid_t(1000), fspp::gid_t(1000));
        this->setAtimeOlderThanMtime("/mydir");
        return [dir = std::move(dir)] {
            dir->children();
        };
    };
    this->testBuilder().withNoatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mydir", operation(), {this->ExpectDoesntUpdateAnyTimestamps});
    }).withStrictatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mydir", operation(), {this->ExpectUpdatesAccessTimestamp, this->ExpectDoesntUpdateModificationTimestamp, this->ExpectDoesntUpdateMetadataTimestamp});
    }).withRelatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mydir", operation(), {this->ExpectUpdatesAccessTimestamp, this->ExpectDoesntUpdateModificationTimestamp, this->ExpectDoesntUpdateMetadataTimestamp});
    }).withNodiratimeRelatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mydir", operation(), {this->ExpectDoesntUpdateAnyTimestamps});
    }).withNodiratimeStrictatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mydir", operation(), {this->ExpectDoesntUpdateAnyTimestamps});
    });
}

TYPED_TEST_P(FsppDirTest_Timestamps, givenAtimeNewerThanMtime_children_nonempty) {
    auto operation = [this] {
        auto dir = this->CreateDir("/mydir");
        dir->createAndOpenFile("filename", fspp::mode_t().addFileFlag(), fspp::uid_t(1000), fspp::gid_t(1000));
        this->setAtimeNewerThanMtime("/mydir");
        return [dir = std::move(dir)] {
            dir->children();
        };
    };
    this->testBuilder().withNoatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mydir", operation(), {this->ExpectDoesntUpdateAnyTimestamps});
    }).withStrictatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mydir", operation(), {this->ExpectUpdatesAccessTimestamp, this->ExpectDoesntUpdateModificationTimestamp, this->ExpectDoesntUpdateMetadataTimestamp});
    }).withRelatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mydir", operation(), {this->ExpectDoesntUpdateAnyTimestamps});
    }).withNodiratimeRelatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mydir", operation(), {this->ExpectDoesntUpdateAnyTimestamps});
    }).withNodiratimeStrictatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mydir", operation(), {this->ExpectDoesntUpdateAnyTimestamps});
    });
}

TYPED_TEST_P(FsppDirTest_Timestamps, givenAtimeNewerThanMtimeButBeforeYesterday_children_nonempty) {
    auto operation = [this] {
        auto dir = this->CreateDir("/mydir");
        dir->createAndOpenFile("filename", fspp::mode_t().addFileFlag(), fspp::uid_t(1000), fspp::gid_t(1000));
        this->setAtimeNewerThanMtimeButBeforeYesterday("/mydir");
        return [dir = std::move(dir)] {
            dir->children();
        };
    };
    this->testBuilder().withNoatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mydir", operation(), {this->ExpectDoesntUpdateAnyTimestamps});
    }).withStrictatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mydir", operation(), {this->ExpectUpdatesAccessTimestamp, this->ExpectDoesntUpdateModificationTimestamp, this->ExpectDoesntUpdateMetadataTimestamp});
    }).withRelatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mydir", operation(), {this->ExpectUpdatesAccessTimestamp, this->ExpectDoesntUpdateModificationTimestamp, this->ExpectDoesntUpdateMetadataTimestamp});
    }).withNodiratimeRelatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mydir", operation(), {this->ExpectDoesntUpdateAnyTimestamps});
    }).withNodiratimeStrictatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mydir", operation(), {this->ExpectDoesntUpdateAnyTimestamps});
    });
}

/* TODO Re-enable this test once the root dir handles timestamps correctly
TYPED_TEST_P(FsppDirTest_Timestamps, givenAtimeOlderThanMtime_children_nonempty_inRootDir) {
    auto operation = [this] {
        auto dir = this->LoadDir("/");
        dir->createAndOpenFile("filename", fspp::mode_t().addFileFlag(), fspp::uid_t(1000), fspp::gid_t(1000));
        this->setAtimeOlderThanMtime("/");
        return [dir = std::move(dir)] {
            dir->children();
        };
    };
    this->testBuilder().withNoatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/", operation(), {this->ExpectDoesntUpdateAnyTimestamps});
    }).withStrictatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/", operation(), {this->ExpectUpdatesAccessTimestamp, this->ExpectDoesntUpdateModificationTimestamp, this->ExpectDoesntUpdateMetadataTimestamp});
    }).withRelatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/", operation(), {this->ExpectUpdatesAccessTimestamp, this->ExpectDoesntUpdateModificationTimestamp, this->ExpectDoesntUpdateMetadataTimestamp});
    }).withNodiratimeRelatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/", operation(), {this->ExpectDoesntUpdateAnyTimestamps});
    }).withNodiratimeStrictatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/", operation(), {this->ExpectDoesntUpdateAnyTimestamps});
    });
}

TYPED_TEST_P(FsppDirTest_Timestamps, givenAtimeNewerThanMtime_children_nonempty_inRootDir) {
    auto operation = [this] {
        auto dir = this->LoadDir("/");
        dir->createAndOpenFile("filename", fspp::mode_t().addFileFlag(), fspp::uid_t(1000), fspp::gid_t(1000));
        this->setAtimeNewerThanMtime("/");
        return [dir = std::move(dir)] {
            dir->children();
        };
    };
    this->testBuilder().withNoatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/", operation(), {this->ExpectDoesntUpdateAnyTimestamps});
    }).withStrictatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/", operation(), {this->ExpectUpdatesAccessTimestamp, this->ExpectDoesntUpdateModificationTimestamp, this->ExpectDoesntUpdateMetadataTimestamp});
    }).withRelatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/", operation(), {this->ExpectDoesntUpdateAnyTimestamps});
    }).withNodiratimeRelatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/", operation(), {this->ExpectDoesntUpdateAnyTimestamps});
    }).withNodiratimeStrictatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/", operation(), {this->ExpectDoesntUpdateAnyTimestamps});
    });
}

TYPED_TEST_P(FsppDirTest_Timestamps, givenAtimeNewerThanMtimeButBeforeYesterday_children_nonempty_inRootDir) {
    auto operation = [this] {
        auto dir = this->LoadDir("/");
        dir->createAndOpenFile("filename", fspp::mode_t().addFileFlag(), fspp::uid_t(1000), fspp::gid_t(1000));
        this->setAtimeNewerThanMtimeButBeforeYesterday("/");
        return [dir = std::move(dir)] {
            dir->children();
        };
    };
    this->testBuilder().withNoatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/", operation(), {this->ExpectDoesntUpdateAnyTimestamps});
    }).withStrictatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/", operation(), {this->ExpectUpdatesAccessTimestamp, this->ExpectDoesntUpdateModificationTimestamp, this->ExpectDoesntUpdateMetadataTimestamp});
    }).withRelatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/", operation(), {this->ExpectUpdatesAccessTimestamp, this->ExpectDoesntUpdateModificationTimestamp, this->ExpectDoesntUpdateMetadataTimestamp});
    }).withNodiratimeRelatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/", operation(), {this->ExpectDoesntUpdateAnyTimestamps});
    }).withNodiratimeStrictatime([&] {
        this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/", operation(), {this->ExpectDoesntUpdateAnyTimestamps});
    });
}
*/

template<class ConcreteFileSystemTestFixture>
class FsppDirTest_Timestamps_Entries: public FsppNodeTest<ConcreteFileSystemTestFixture>, public TimestampTestUtils<ConcreteFileSystemTestFixture> {
public:
    void Test_deleteChild() {
        auto operation = [this] {
            auto dir = this->CreateDir("/mydir");
            auto child = this->CreateNode("/mydir/childname");
            return [child = std::move(child)] {
                child->remove();
            };
        };
        this->testBuilder().withAnyAtimeConfig([&] {
            this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mydir", operation(), {this->ExpectDoesntUpdateAccessTimestamp, this->ExpectUpdatesModificationTimestamp, this->ExpectUpdatesMetadataTimestamp});
        });
    }

    /* TODO Re-enable this test once the root dir handles timestamps correctly
    void Test_deleteChild_inRootDir() {
        auto operation = [this] {
            auto dir = this->LoadDir("/");
            auto child = this->CreateNode("/mydir/childname");
            return [child = std::move(child)] {
                child->remove();
            };
        };
        this->testBuilder().withAnyAtimeConfig([&] {
            this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/", operation(), {this->ExpectDoesntUpdateAccessTimestamp, this->ExpectUpdatesModificationTimestamp, this->ExpectUpdatesMetadataTimestamp});
        });
    }
    */

    void Test_renameChild() {
        auto operation = [this] {
            auto dir = this->CreateDir("/mydir");
            auto child = this->CreateNode("/mydir/childname");
            return [child = std::move(child)] {
                child->rename("/mydir/mychild");
            };
        };
        this->testBuilder().withAnyAtimeConfig([&]{
            this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mydir", operation(), {this->ExpectDoesntUpdateAccessTimestamp, this->ExpectUpdatesModificationTimestamp, this->ExpectUpdatesMetadataTimestamp});
        });
    }

    /* TODO Re-enable this test once the root dir handles timestamps correctly
    void Test_renameChild_inRootDir() {
        auto operation = [this] {
            auto dir = this->LoadDir("/");
            auto child = this->CreateNode("/childname");
            return [child = std::move(child)] {
                child->rename("/mychild");
            };
        };
        this->testBuilder().withAnyAtimeConfig([&]{
            this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/", operation(), {this->ExpectDoesntUpdateAccessTimestamp, this->ExpectUpdatesModificationTimestamp, this->ExpectUpdatesMetadataTimestamp});
        });
    }
    */

    void Test_moveChildIn() {
        auto operation = [this] {
            auto sourceDir = this->CreateDir("/sourcedir");
            auto child = this->CreateNode("/sourcedir/childname");
            auto dir = this->CreateDir("/mydir");
            return [child = std::move(child)] {
                child->rename("/mydir/mychild");
            };
        };
        this->testBuilder().withAnyAtimeConfig([&] {
            this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mydir", operation(), {this->ExpectDoesntUpdateAccessTimestamp, this->ExpectUpdatesModificationTimestamp, this->ExpectUpdatesMetadataTimestamp});
        });
    }

    /* TODO Re-enable this test once the root dir handles timestamps correctly
    void Test_moveChildIn_inRootDir() {
        auto operation = [this] {
            auto sourceDir = this->CreateDir("/sourcedir");
            auto child = this->CreateNode("/sourcedir/childname");
            auto dir = this->LoadDir("/");
            return [child = std::move(child)] {
                child->rename("/mychild");
            };
        };
        this->testBuilder().withAnyAtimeConfig([&] {
            this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mydir", operation(), {this->ExpectDoesntUpdateAccessTimestamp, this->ExpectUpdatesModificationTimestamp, this->ExpectUpdatesMetadataTimestamp});
        });
    }
    */

    void Test_moveChildOut() {
        auto operation = [this] {
            auto dir = this->CreateDir("/mydir");
            auto child = this->CreateNode("/mydir/childname");
            this->CreateDir("/targetdir");
            return [child = std::move(child)] {
                child->rename("/targetdir/mychild");
            };
        };
        this->testBuilder().withAnyAtimeConfig([&] {
            this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mydir", operation(), {this->ExpectDoesntUpdateAccessTimestamp, this->ExpectUpdatesModificationTimestamp, this->ExpectUpdatesMetadataTimestamp});
        });
    }

    /* TODO Re-enable this test once the root dir handles timestamps correctly
    void Test_moveChildOut_inRootDir() {
        auto operation = [this] {
            auto dir = this->LoadDir("/");
            auto child = this->CreateNode("/childname");
            this->CreateDir("/targetdir");
            return [child = std::move(child)] {
                child->rename("/targetdir/mychild");
            };
        };
        this->testBuilder().withAnyAtimeConfig([&] {
            this->EXPECT_OPERATION_UPDATES_TIMESTAMPS_AS("/mydir", operation(), {this->ExpectDoesntUpdateAccessTimestamp, this->ExpectUpdatesModificationTimestamp, this->ExpectUpdatesMetadataTimestamp});
        });
    }
    */
};

REGISTER_TYPED_TEST_SUITE_P(FsppDirTest_Timestamps,
   createAndOpenFile,
   createAndOpenFile_TimestampsOfCreatedFile,
   createDir,
   createDir_TimestampsOfCreatedDir,
   createSymlink,
   createSymlink_TimestampsOfCreatedSymlink,
   givenAtimeNewerThanMtime_children_empty,
   givenAtimeOlderThanMtime_children_empty,
   givenAtimeNewerThanMtimeButBeforeYesterday_children_empty,
   givenAtimeNewerThanMtime_children_nonempty,
   givenAtimeOlderThanMtime_children_nonempty,
   givenAtimeNewerThanMtimeButBeforeYesterday_children_nonempty
);

REGISTER_NODE_TEST_SUITE(FsppDirTest_Timestamps_Entries,
   deleteChild,
   renameChild,
   moveChildIn,
   moveChildOut
);

#endif
