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

class Load_Test_With_Filesystem : public C_Library_Test {
public:
    Load_Test_With_Filesystem(): basedir(), externalconfig(false) {}

    void create_filesystem(const bf::path &basedir, const optional<bf::path> &configfile_path = none) {
        bf::path actual_configfile_path;
        if (configfile_path == none) {
            actual_configfile_path = basedir / "cryfs.config";
        } else {
            actual_configfile_path = *configfile_path;
        }
        auto configfile = create_configfile(actual_configfile_path);
        auto blockstore = make_unique_ref<OnDiskBlockStore>(basedir);
        CryDevice device(std::move(configfile), std::move(blockstore));
    }

    CryConfigFile create_configfile(const bf::path &configfile_path) {
        CryConfig config;
        config.SetCipher("aes-256-gcm");
        config.SetEncryptionKey(AES256_GCM::CreateKey(Random::PseudoRandom()).ToString());
        config.SetRootBlob("");
        config.SetVersion(version::VERSION_STRING);

        return CryConfigFile::create(configfile_path, std::move(config), PASSWORD, SCrypt::TestSettings);
    }

    CryConfigFile create_configfile_for_incompatible_cryfs_version(const bf::path &configfile_path) {
        CryConfig config;
        config.SetCipher("aes-256-gcm");
        config.SetEncryptionKey(AES256_GCM::CreateKey(Random::PseudoRandom()).ToString());
        config.SetRootBlob("");
        config.SetVersion("0.8.0");

        return CryConfigFile::create(configfile_path, std::move(config), PASSWORD, SCrypt::TestSettings);
    }

    void remove_all_blocks_in(const bf::path &dir) {
        for (bf::directory_iterator iter(dir); iter != bf::directory_iterator(); ++iter) {
            if (iter->path().filename() != "cryfs.config") {
                bf::remove(iter->path());
            }
        }
    }

    void set_basedir(const optional<bf::path> &_basedir = none) {
        bf::path actual_basedir = basedir.path();
        if (_basedir != none) {
            actual_basedir = *_basedir;
        }
        EXPECT_SUCCESS(cryfs_load_set_basedir(context, actual_basedir.native().c_str(), actual_basedir.native().size()));
    }

    void set_password(const string &password = PASSWORD) {
        EXPECT_SUCCESS(cryfs_load_set_password(context, password.c_str(), password.size()));
    }

    void set_externalconfig() {
        EXPECT_SUCCESS(cryfs_load_set_externalconfig(context, externalconfig.path().native().c_str(), externalconfig.path().native().size()));
    }

    TempDir basedir;
    TempFile externalconfig;
    static const std::string PASSWORD;
};

const std::string Load_Test_With_Filesystem::PASSWORD = "mypassword";

TEST_F(Load_Test_With_Filesystem, setup) {
        //Do nothing, just test that the file system can be setup properly
}

TEST_F(Load_Test_With_Filesystem, load) {
    create_filesystem(basedir.path());
    set_basedir();
    set_password();
    EXPECT_LOAD_SUCCESS();
}

TEST_F(Load_Test_With_Filesystem, load_withexternalconfig) {
    create_filesystem(basedir.path(), externalconfig.path());
    set_basedir();
    set_externalconfig();
    set_password();
    EXPECT_LOAD_SUCCESS();
}

TEST_F(Load_Test_With_Filesystem, load_wrongpassword) {
    create_filesystem(basedir.path());
    set_basedir();
    set_password("wrong_password");
    EXPECT_LOAD_ERROR(cryfs_error_DECRYPTION_FAILED);
}

TEST_F(Load_Test_With_Filesystem, load_wrongpassword_withexternalconfig) {
    create_filesystem(basedir.path(), externalconfig.path());
    set_basedir();
    set_externalconfig();
    set_password("wrong_password");
    EXPECT_LOAD_ERROR(cryfs_error_DECRYPTION_FAILED);
}

TEST_F(Load_Test_With_Filesystem, load_missingrootblob) {
    create_filesystem(basedir.path());
    remove_all_blocks_in(basedir.path());
    set_basedir();
    set_password();
    EXPECT_LOAD_ERROR(cryfs_error_FILESYSTEM_INVALID);
}

TEST_F(Load_Test_With_Filesystem, load_missingrootblob_withexternalconfig) {
    create_filesystem(basedir.path(), externalconfig.path());
    remove_all_blocks_in(basedir.path());
    set_basedir();
    set_externalconfig();
    set_password();
    EXPECT_LOAD_ERROR(cryfs_error_FILESYSTEM_INVALID);
}

TEST_F(Load_Test_With_Filesystem, load_missingconfigfile) {
    create_filesystem(basedir.path(), externalconfig.path());
    bf::remove(basedir.path() / "cryfs.config");
    set_basedir();
    set_password();
    EXPECT_LOAD_ERROR(cryfs_error_CONFIGFILE_DOESNT_EXIST);
}

TEST_F(Load_Test_With_Filesystem, load_missingconfigfile_withexternalconfig) {
    create_filesystem(basedir.path(), externalconfig.path());
    set_basedir();
    set_externalconfig();
    externalconfig.remove();
    set_password();
    EXPECT_LOAD_ERROR(cryfs_error_CONFIGFILE_DOESNT_EXIST);
}

TEST_F(Load_Test_With_Filesystem, load_incompatible_version) {
    create_filesystem(basedir.path());
    create_configfile_for_incompatible_cryfs_version(externalconfig.path());
    set_basedir();
    set_externalconfig();
    set_password();
    EXPECT_LOAD_ERROR(cryfs_error_FILESYSTEM_INCOMPATIBLE_VERSION);
}
