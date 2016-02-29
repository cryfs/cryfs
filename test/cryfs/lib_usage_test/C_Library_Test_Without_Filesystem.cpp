#include "testutils/C_Library_Test.h"

using std::string;
using cpputils::TempDir;
using cpputils::TempFile;
namespace bf = boost::filesystem;

class C_Library_Test_Without_Filesystem : public C_Library_Test {
public:
    const string NONEXISTENT_PATH = "/some/nonexistent/path";
    const string PASSWORD = "mypassword";
    TempFile _existing_file;
    const string EXISTING_FILE = _existing_file.path().native();
    TempDir _existing_dir;
    const string EXISTING_DIR = _existing_dir.path().native();
};

TEST_F(C_Library_Test_Without_Filesystem, init_and_free) {
    // Don't do anything in here.
    // This tests that the constructor successfully initializes the context and it can be freed in the destructor.
}

TEST_F(C_Library_Test_Without_Filesystem, basedir_doesnt_exist) {
    EXPECT_EQ(cryfs_error_BASEDIR_DOESNT_EXIST, cryfs_load_set_basedir(context, NONEXISTENT_PATH.c_str(), NONEXISTENT_PATH.size()));
}

TEST_F(C_Library_Test_Without_Filesystem, basedir_valid) {
    EXPECT_EQ(cryfs_success, cryfs_load_set_basedir(context, EXISTING_DIR.c_str(), EXISTING_DIR.size()));
}

TEST_F(C_Library_Test_Without_Filesystem, externalconfig_doesnt_exist) {
    EXPECT_EQ(cryfs_error_CONFIGFILE_DOESNT_EXIST, cryfs_load_set_externalconfig(context, NONEXISTENT_PATH.c_str(), NONEXISTENT_PATH.size()));
}

TEST_F(C_Library_Test_Without_Filesystem, externalconfig_valid) {
    EXPECT_EQ(cryfs_success, cryfs_load_set_externalconfig(context, EXISTING_FILE.c_str(), EXISTING_FILE.size()));
}

TEST_F(C_Library_Test_Without_Filesystem, password) {
    EXPECT_EQ(cryfs_success, cryfs_load_set_password(context, PASSWORD.c_str(), PASSWORD.size()));
}

TEST_F(C_Library_Test_Without_Filesystem, load_without_basedir) {
    cryfs_mount_handle *handle = nullptr;
    EXPECT_EQ(cryfs_error_BASEDIR_NOT_SET, cryfs_load(context, &handle));
    EXPECT_EQ(nullptr, handle);
}

TEST_F(C_Library_Test_Without_Filesystem, load_with_invalid_basedir) {
    EXPECT_FAIL(cryfs_load_set_basedir(context, NONEXISTENT_PATH.c_str(), NONEXISTENT_PATH.size()));
    cryfs_mount_handle *handle = nullptr;
    EXPECT_EQ(cryfs_error_BASEDIR_NOT_SET, cryfs_load(context, &handle));
    EXPECT_EQ(nullptr, handle);
}

TEST_F(C_Library_Test_Without_Filesystem, load_without_password) {
    EXPECT_SUCCESS(cryfs_load_set_basedir(context, EXISTING_DIR.c_str(), EXISTING_DIR.size()));
    cryfs_mount_handle *handle = nullptr;
    EXPECT_EQ(cryfs_error_PASSWORD_NOT_SET, cryfs_load(context, &handle));
    EXPECT_EQ(nullptr, handle);
}

TEST_F(C_Library_Test_Without_Filesystem, load_emptybasedir) {
    EXPECT_SUCCESS(cryfs_load_set_basedir(context, EXISTING_DIR.c_str(), EXISTING_DIR.size()));
    EXPECT_SUCCESS(cryfs_load_set_password(context, PASSWORD.c_str(), PASSWORD.size()));
    cryfs_mount_handle *handle = nullptr;
    EXPECT_EQ(cryfs_error_FILESYSTEM_NOT_FOUND, cryfs_load(context, &handle));
    EXPECT_EQ(nullptr, handle);
}

TEST_F(C_Library_Test_Without_Filesystem, load_emptybasedir_withexternalconfig) {
    EXPECT_SUCCESS(cryfs_load_set_basedir(context, EXISTING_DIR.c_str(), EXISTING_DIR.size()));
    EXPECT_SUCCESS(cryfs_load_set_externalconfig(context, EXISTING_FILE.c_str(), EXISTING_FILE.size()));
    EXPECT_SUCCESS(cryfs_load_set_password(context, PASSWORD.c_str(), PASSWORD.size()));
    cryfs_mount_handle *handle = nullptr;
    EXPECT_EQ(cryfs_error_FILESYSTEM_NOT_FOUND, cryfs_load(context, &handle));
    EXPECT_EQ(nullptr, handle);
}

//TODO Add test cases for all existing error codes (here or in C_Library_Test_With_Filesystem)
