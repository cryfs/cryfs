#include "testutils/C_Library_Test.h"

using std::string;
using cpputils::TempDir;
using cpputils::TempFile;
namespace bf = boost::filesystem;

class Load_Test_Without_Filesystem : public C_Library_Test {
public:
    const string INVALID_PATH = "pathname_with_some_invalid_characters_$% ä*.\\\"[]:;|=,";
    const string NONEXISTENT_PATH = "/some/nonexistent/path";
    const string PASSWORD = "mypassword";
    TempFile _existing_file;
    const string EXISTING_FILE = _existing_file.path().native();
    TempDir _existing_dir;
    const string EXISTING_DIR = _existing_dir.path().native();

    void set_existing_basedir() {
        EXPECT_SUCCESS(cryfs_load_set_basedir(context, EXISTING_DIR.c_str(), EXISTING_DIR.size()));
    }

    void set_externalconfig(const bf::path &configPath) {
        EXPECT_SUCCESS(cryfs_load_set_externalconfig(context, configPath.native().c_str(), configPath.native().size()));
    }

    void set_password() {
        EXPECT_SUCCESS(cryfs_load_set_password(context, PASSWORD.c_str(), PASSWORD.size()));
    }
};

TEST_F(Load_Test_Without_Filesystem, init_and_free) {
    // Don't do anything in here.
    // This tests that the constructor successfully initializes the context and it can be freed in the destructor.
}

TEST_F(Load_Test_Without_Filesystem, init_unsupported_api_version) {
  cryfs_load_context *context;
  EXPECT_EQ(cryfs_error_UNSUPPORTED_API_VERSION, cryfs_load_init(2, &context));
  cryfs_load_free(context); // Test that people can call cryfs_load_free after an error in cryfs_load_init
}

TEST_F(Load_Test_Without_Filesystem, basedir_doesnt_exist) {
    EXPECT_EQ(cryfs_error_BASEDIR_DOESNT_EXIST, cryfs_load_set_basedir(context, NONEXISTENT_PATH.c_str(), NONEXISTENT_PATH.size()));
}

TEST_F(Load_Test_Without_Filesystem, basedir_invalid) {
    EXPECT_EQ(cryfs_error_BASEDIR_DOESNT_EXIST, cryfs_load_set_basedir(context, INVALID_PATH.c_str(), INVALID_PATH.size()));
}

TEST_F(Load_Test_Without_Filesystem, basedir_is_file) {
    EXPECT_EQ(cryfs_error_BASEDIR_INACCESSIBLE, cryfs_load_set_basedir(context, EXISTING_FILE.c_str(), EXISTING_FILE.size()));
}

TEST_F(Load_Test_Without_Filesystem, basedir_not_readable) {
    chmod(EXISTING_DIR.c_str(), S_IWUSR | S_IXUSR | S_IWGRP | S_IXGRP | S_IWOTH | S_IXOTH);
    EXPECT_EQ(cryfs_error_BASEDIR_INACCESSIBLE, cryfs_load_set_basedir(context, EXISTING_DIR.c_str(), EXISTING_DIR.size()));
}

TEST_F(Load_Test_Without_Filesystem, basedir_not_writeble) {
    chmod(EXISTING_DIR.c_str(), S_IRUSR | S_IXUSR | S_IRGRP | S_IXGRP | S_IROTH | S_IXOTH);
    EXPECT_EQ(cryfs_error_BASEDIR_INACCESSIBLE, cryfs_load_set_basedir(context, EXISTING_DIR.c_str(), EXISTING_DIR.size()));
}

TEST_F(Load_Test_Without_Filesystem, basedir_not_enterable) {
    chmod(EXISTING_DIR.c_str(), S_IRUSR | S_IWUSR | S_IRGRP | S_IWGRP | S_IROTH | S_IWOTH);
    EXPECT_EQ(cryfs_error_BASEDIR_INACCESSIBLE, cryfs_load_set_basedir(context, EXISTING_DIR.c_str(), EXISTING_DIR.size()));
}

TEST_F(Load_Test_Without_Filesystem, basedir_valid) {
    EXPECT_EQ(cryfs_success, cryfs_load_set_basedir(context, EXISTING_DIR.c_str(), EXISTING_DIR.size()));
}

TEST_F(Load_Test_Without_Filesystem, externalconfig_doesnt_exist) {
    EXPECT_EQ(cryfs_error_CONFIGFILE_DOESNT_EXIST, cryfs_load_set_externalconfig(context, NONEXISTENT_PATH.c_str(), NONEXISTENT_PATH.size()));
}

TEST_F(Load_Test_Without_Filesystem, externalconfig_invalid) {
    EXPECT_EQ(cryfs_error_CONFIGFILE_DOESNT_EXIST, cryfs_load_set_externalconfig(context, INVALID_PATH.c_str(), INVALID_PATH.size()));
}

TEST_F(Load_Test_Without_Filesystem, externalconfig_is_dir) {
  EXPECT_EQ(cryfs_error_CONFIGFILE_NOT_READABLE, cryfs_load_set_externalconfig(context, EXISTING_DIR.c_str(), EXISTING_DIR.size()));
}

TEST_F(Load_Test_Without_Filesystem, externalconfig_not_readable) {
  chmod(EXISTING_FILE.c_str(), S_IWUSR | S_IXUSR | S_IWGRP | S_IXGRP | S_IWOTH | S_IXOTH);
  EXPECT_EQ(cryfs_error_CONFIGFILE_NOT_READABLE, cryfs_load_set_externalconfig(context, EXISTING_FILE.c_str(), EXISTING_FILE.size()));
}

TEST_F(Load_Test_Without_Filesystem, externalconfig_valid) {
    EXPECT_EQ(cryfs_success, cryfs_load_set_externalconfig(context, EXISTING_FILE.c_str(), EXISTING_FILE.size()));
}

TEST_F(Load_Test_Without_Filesystem, password) {
    EXPECT_EQ(cryfs_success, cryfs_load_set_password(context, PASSWORD.c_str(), PASSWORD.size()));
}

TEST_F(Load_Test_Without_Filesystem, load_without_basedir) {
    EXPECT_LOAD_ERROR(cryfs_error_BASEDIR_NOT_SET);
}

TEST_F(Load_Test_Without_Filesystem, load_with_invalid_basedir) {
    EXPECT_FAIL(cryfs_load_set_basedir(context, NONEXISTENT_PATH.c_str(), NONEXISTENT_PATH.size()));
    EXPECT_LOAD_ERROR(cryfs_error_BASEDIR_NOT_SET);
}

TEST_F(Load_Test_Without_Filesystem, load_without_password) {
    set_existing_basedir();
    EXPECT_LOAD_ERROR(cryfs_error_PASSWORD_NOT_SET);
}

TEST_F(Load_Test_Without_Filesystem, load_emptybasedir) {
    set_existing_basedir();
    set_password();
    EXPECT_LOAD_ERROR(cryfs_error_CONFIGFILE_DOESNT_EXIST);
}

TEST_F(Load_Test_Without_Filesystem, load_emptybasedir_withexternalconfig) {
    set_existing_basedir();
    set_externalconfig(_existing_file.path());
    set_password();
    EXPECT_LOAD_ERROR(cryfs_error_DECRYPTION_FAILED);
}
