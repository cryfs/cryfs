#include <cryfs/impl/config/CryConfig.h>
#include <cryfs/impl/config/CryConfigFile.h>
#include <cryfs/impl/filesystem/CryDevice.h>
#include <blockstore/implementations/ondisk/OnDiskBlockStore.h>
#include "testutils/C_Library_Test.h"
#include <gitversion/version.h>

using cryfs::CryConfig;
using cryfs::CryConfigFile;
using cryfs::CryDevice;
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

class C_Library_Test_With_Filesystem : public C_Library_Test {
public:
    C_Library_Test_With_Filesystem(): basedir(), externalconfig(false) {}

    void create_filesystem(const bf::path &basedir, const optional<bf::path> &configfile_path = none) {
            CryConfig config;
            config.SetCipher("aes-256-gcm");
            config.SetEncryptionKey(AES256_GCM::CreateKey(Random::PseudoRandom()).ToString());
            config.SetRootBlob("");
            config.SetVersion(version::VERSION_STRING);

            bf::path actual_configfile_path;
            if (configfile_path == none) {
                actual_configfile_path = basedir / "cryfs.config";
            } else {
                actual_configfile_path = *configfile_path;
            }
            CryConfigFile configfile = CryConfigFile::create(actual_configfile_path, std::move(config), PASSWORD, SCrypt::TestSettings);
            //auto blockstore = make_unique_ref<OnDiskBlockStore>(basedir);
            //CryDevice(std::move(configfile), std::move(blockstore));
    }

    TempDir basedir;
    TempFile externalconfig;
    static const std::string PASSWORD;
};

const std::string C_Library_Test_With_Filesystem::PASSWORD = "mypassword";

TEST_F(C_Library_Test_With_Filesystem, setup) {
        //Do nothing, just test that the file system can be setup properly
}

TEST_F(C_Library_Test_With_Filesystem, load) {
    create_filesystem(basedir.path());
    EXPECT_SUCCESS(cryfs_load_set_basedir(context, basedir.path().native().c_str(), basedir.path().native().size()));
    EXPECT_SUCCESS(cryfs_load_set_password(context, PASSWORD.c_str(), PASSWORD.size()));
    cryfs_mount_handle *handle = nullptr;
    EXPECT_EQ(cryfs_success, cryfs_load(context, &handle));
    EXPECT_NE(nullptr, handle);
}

TEST_F(C_Library_Test_With_Filesystem, load_withexternalconfig) {
    create_filesystem(basedir.path(), externalconfig.path());
    EXPECT_SUCCESS(cryfs_load_set_basedir(context, basedir.path().native().c_str(), basedir.path().native().size()));
    EXPECT_SUCCESS(cryfs_load_set_externalconfig(context, externalconfig.path().native().c_str(), externalconfig.path().native().size()));
    EXPECT_SUCCESS(cryfs_load_set_password(context, PASSWORD.c_str(), PASSWORD.size()));
    cryfs_mount_handle *handle = nullptr;
    EXPECT_EQ(cryfs_success, cryfs_load(context, &handle));
    EXPECT_NE(nullptr, handle);
}

/*
//TODO Fix this behavior in the library so that this test case passes
TEST_F(C_Library_Test_With_Filesystem, load_wrongpassword) {
    const std::string WRONG_PASSWORD = "wrong_password";
    create_filesystem(basedir.path());
    EXPECT_SUCCESS(cryfs_load_set_basedir(context, basedir.path().native().c_str(), basedir.path().native().size()));
    EXPECT_SUCCESS(cryfs_load_set_password(context, WRONG_PASSWORD.c_str(), WRONG_PASSWORD.size()));
    cryfs_mount_handle *handle = nullptr;
    EXPECT_EQ(cryfs_error_DECRYPTION_FAILED, cryfs_load(context, &handle));
    EXPECT_EQ(nullptr, handle);
}
*/

//TODO Test loading invalid file system (i.e. root blob missing) returns cryfs_error_FILESYSTEM_INVALID
//TODO Add test cases loading file systems with an incompatible version returns cryfs_error_FILESYSTEM_INCOMPATIBLE_VERSION
