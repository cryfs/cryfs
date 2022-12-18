// This test tests various ways of renaming files in nested directory structures.
// It tests both that the rename operation succeeds, and that it doesn't deadlock
// This is important because our CryNode implementation accesses multiple blobs
// (source, source_parent, target_parent, target_grandparent) and if any of those
// overlap, we need to make sure that we don't deadlock by trying to load them
// at the same time. This is also why these tests nest quite deeply.

#include <gtest/gtest.h>
#include "testutils/CryTestBase.h"
#include <cryfs/impl/filesystem/CryDir.h>
#include <cryfs/impl/filesystem/CryFile.h>
#include <cryfs/impl/filesystem/CryOpenFile.h>
#include <fspp/fs_interface/FuseErrnoException.h>
#include <boost/algorithm/string/predicate.hpp>

using ::testing::Combine;
using ::testing::ValuesIn;
using ::testing::TestWithParam;
using std::vector;
namespace bf = boost::filesystem;
using fspp::fuse::FuseErrnoException;

namespace {
    vector<bf::path> SourceDirs() {
        return {
            "/",
            "/a1",
            "/a1/b1",
            "/a1/b1/c1",
            "/a1/b1/c1/d1",
            "/a1/b1/c1/d1/e1",
            "/a1/b1/c1/d1/e1/f1",
        };
    }

    vector<bf::path> DestDirs() {
        auto result = SourceDirs();
        result.push_back("/a2");
        result.push_back("/a2/b");
        result.push_back("/a2/b/c");
        result.push_back("/a2/b/c/d");
        result.push_back("/a2/b/c/d/e");
        result.push_back("/a2/b/c/d/e/f");
        result.push_back("/a1/b2");
        result.push_back("/a1/b2/c");
        result.push_back("/a1/b2/c/d");
        result.push_back("/a1/b2/c/d/e");
        result.push_back("/a1/b2/c/d/e/f");
        result.push_back("/a1/b1/c2");
        result.push_back("/a1/b1/c2/d");
        result.push_back("/a1/b1/c2/d/e");
        result.push_back("/a1/b1/c2/d/e/f");
        result.push_back("/a1/b1/c1/d2");
        result.push_back("/a1/b1/c1/d2/e");
        result.push_back("/a1/b1/c1/d2/e/f");
        result.push_back("/a1/b1/c1/d1/e2");
        result.push_back("/a1/b1/c1/d1/e2/f");
        result.push_back("/a1/b1/c1/d1/e1/f2");
        return result;
    }
}

class CryNodeTest_RenameNested : public TestWithParam<std::tuple<bf::path, bf::path>>, public CryTestBase {
public:
    void CreateDirs() {
        for (const auto& dir : SourceDirs()) {
            if (dir != "/") {
                CreateDir(dir);
            }
        }
    }

    void create_path_if_not_exists(const bf::path& path) {
        if (!Exists(path)) {
            if (path.has_parent_path()) {
                create_path_if_not_exists(path.parent_path());
            }
            CreateDir(path);
        }

    }

    void expect_rename_succeeds(const bf::path& source_path, const bf::path& dest_path) {
        auto source = device().Load(source_path).value();
        source->rename(dest_path);
        // TODO Test that rename was successful
    }

    void expect_rename_fails(const bf::path& source_path, const bf::path& dest_path, int expected_errno) {
        auto source = device().Load(source_path).value();
        try {
            source->rename(dest_path);
            ASSERT(false, "Expected throw FuseErrnoException(" + std::to_string(expected_errno) + " but didn't throw");
        } catch (const FuseErrnoException& e) {
            ASSERT_EQ(expected_errno, e.getErrno());
        }
        // TODO Test rename wasn't successful
    }
};

INSTANTIATE_TEST_SUITE_P(All, CryNodeTest_RenameNested, Combine(ValuesIn(SourceDirs()), ValuesIn(DestDirs())));

TEST_P(CryNodeTest_RenameNested, Rename) {
    CreateDirs();

    auto source_path = std::get<0>(GetParam());
    auto dest_path = std::get<1>(GetParam());

    if (dest_path.has_parent_path()) {
        create_path_if_not_exists(dest_path.parent_path());
    }

    if (source_path == "/" || dest_path == "/") {
        expect_rename_fails(source_path, dest_path, EBUSY);
    } else if (source_path == dest_path) {
        expect_rename_succeeds(source_path, dest_path);
    } else if (boost::starts_with(source_path, dest_path)) {
        expect_rename_fails(source_path, dest_path, ENOTEMPTY);
    } else if (boost::starts_with(dest_path, source_path)) {
        expect_rename_fails(source_path, dest_path, EINVAL);
    } else {
        expect_rename_succeeds(source_path, dest_path);
    }
}
