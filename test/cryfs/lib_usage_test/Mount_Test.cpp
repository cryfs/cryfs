#include <cryfs/impl/config/CryConfig.h>
#include <cryfs/impl/config/CryConfigFile.h>
#include <cryfs/impl/filesystem/CryDevice.h>
#include <blockstore/implementations/ondisk/OnDiskBlockStore.h>
#include "testutils/C_Library_Test.h"
#include <gitversion/version.h>

using cryfs::CryConfig;
using cryfs::CryConfigFile;
using cryfs::CryDevice;
using cryfs::CryCiphers;
using blockstore::ondisk::OnDiskBlockStore;
using cpputils::unique_ref;
using cpputils::make_unique_ref;
using cpputils::TempDir;
using cpputils::TempFile;
using cpputils::Random;
using cpputils::AES256_GCM;
using cpputils::SCrypt;
using boost::optional;
using boost::none;
namespace bf = boost::filesystem;

class Mount_Test : public C_Library_Test {
public:
    TempDir basedir;
    TempDir mountdir;
    static const string PASSWORD;
    static const string NOTEXISTING_DIR;
    static const string INVALID_DIR;

    void create_filesystem(const bf::path &basedir, const string &cipher) {
        auto configfile = create_configfile(basedir / "cryfs.config", cipher);
        auto blockstore = make_unique_ref<OnDiskBlockStore>(basedir);
        CryDevice device(std::move(configfile), std::move(blockstore));
    }

    CryConfigFile create_configfile(const bf::path &configfile_path, const string &cipher) {
        CryConfig config;
        config.SetCipher(cipher);
        config.SetEncryptionKey(CryCiphers::find(cipher).createKey(Random::PseudoRandom()));
        config.SetRootBlob("");
        config.SetVersion(version::VERSION_STRING);

        return CryConfigFile::create(configfile_path, std::move(config), PASSWORD, SCrypt::TestSettings);
    }

    cryfs_mount_handle *create_and_load_filesystem(const string &cipher = "aes-256-gcm") {
        create_filesystem(basedir.path(), cipher);
        EXPECT_EQ(cryfs_success, cryfs_load_set_basedir(context, basedir.path().native().c_str(), basedir.path().native().size()));
        EXPECT_EQ(cryfs_success, cryfs_load_set_password(context, PASSWORD.c_str(), PASSWORD.size()));
        cryfs_mount_handle *handle;
        EXPECT_EQ(cryfs_success, cryfs_load(context, &handle));
        return handle;
    }

    string get_ciphername(cryfs_mount_handle *handle) {
        const char *result = nullptr;
        EXPECT_SUCCESS(cryfs_mount_get_ciphername(handle, &result));
        return result;
    }
};
const string Mount_Test::PASSWORD = "mypassword";
const string Mount_Test::NOTEXISTING_DIR = "/some/notexisting/dir";
const string Mount_Test::INVALID_DIR = "pathname_with_some_invalid_characters_$% ä*.\\\"[]:;|=,";

TEST_F(Mount_Test, setup) {
    // Just test that the test setup works
    create_and_load_filesystem();
}

TEST_F(Mount_Test, get_cipher_1) {
    // Just test that the test setup works
    cryfs_mount_handle *handle = create_and_load_filesystem("aes-256-gcm");
    EXPECT_EQ("aes-256-gcm", get_ciphername(handle));
}

TEST_F(Mount_Test, get_cipher_2) {
    // Just test that the test setup works
    cryfs_mount_handle *handle = create_and_load_filesystem("twofish-256-gcm");
    EXPECT_EQ("twofish-256-gcm", get_ciphername(handle));
}

TEST_F(Mount_Test, set_mountdir_notexisting) {
    // Just test that the test setup works
    cryfs_mount_handle *handle = create_and_load_filesystem();
    EXPECT_EQ(cryfs_error_MOUNTDIR_DOESNT_EXIST, cryfs_mount_set_mountdir(handle, NOTEXISTING_DIR.c_str(), NOTEXISTING_DIR.size()));
}

TEST_F(Mount_Test, set_mountdir_invalid) {
    // Just test that the test setup works
    cryfs_mount_handle *handle = create_and_load_filesystem();
    EXPECT_EQ(cryfs_error_MOUNTDIR_DOESNT_EXIST, cryfs_mount_set_mountdir(handle, INVALID_DIR.c_str(), INVALID_DIR.size()));
}

TEST_F(Mount_Test, set_mountdir_valid) {
    // Just test that the test setup works
    cryfs_mount_handle *handle = create_and_load_filesystem();
    EXPECT_SUCCESS(cryfs_mount_set_mountdir(handle, mountdir.path().native().c_str(), mountdir.path().native().size()));
}

//TODO Test it takes the correct config file when there is an external one specified but there also is one in the directory (for example the test could look at the cipher used to distinguish)