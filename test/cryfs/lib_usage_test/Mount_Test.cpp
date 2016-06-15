#include <cryfs/impl/config/CryConfig.h>
#include <cryfs/impl/config/CryConfigFile.h>
#include <cryfs/impl/filesystem/CryDevice.h>
#include <blockstore/implementations/ondisk/OnDiskBlockStore.h>
#include "testutils/C_Library_Test.h"
#include <gitversion/gitversion.h>
#include "testutils/UnmountAfterTimeout.h"
#include <fstream>

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
using std::shared_ptr;
namespace bf = boost::filesystem;

class Mount_Test : public C_Library_Test {
public:
    cryfs_mount_handle *handle = nullptr;
    TempDir basedir;
    TempDir mountdir;
    TempFile logfile;
    static const string PASSWORD;
    static const string NOTEXISTING_DIR;
    static const string NOTEXISTING_LOGFILE;
    static const string INVALID_PATH;

    void create_filesystem(const bf::path &basedir, const string &cipher) {
        auto configfile = create_configfile(basedir / "cryfs.config", cipher);
        auto blockstore = make_unique_ref<OnDiskBlockStore>(basedir);
        CryDevice device(configfile, std::move(blockstore));
    }

    shared_ptr<CryConfigFile> create_configfile(const bf::path &configfile_path, const string &cipher) {
        CryConfig config;
        config.SetCipher(cipher);
        config.SetEncryptionKey(CryCiphers::find(cipher).createKey(Random::PseudoRandom()));
        config.SetRootBlob("");
        config.SetBlocksizeBytes(32*1024);
        config.SetVersion(gitversion::VersionString());

        return cpputils::to_unique_ptr(CryConfigFile::create(configfile_path, std::move(config), PASSWORD, SCrypt::TestSettings));
    }

    void load_filesystem() {
        handle = nullptr;
        EXPECT_SUCCESS(cryfs_load_set_basedir(context, basedir.path().native().c_str(), basedir.path().native().size()));
        EXPECT_SUCCESS(cryfs_load_set_password(context, PASSWORD.c_str(), PASSWORD.size()));
        EXPECT_SUCCESS(cryfs_load(context, &handle));
        EXPECT_NE(nullptr, handle);
    }

    void create_and_load_filesystem(const string &cipher = "aes-256-gcm") {
        create_filesystem(basedir.path(), cipher);
        load_filesystem();
    }

    string get_ciphername(cryfs_mount_handle *handle) {
        const char *result = nullptr;
        EXPECT_SUCCESS(cryfs_mount_get_ciphername(handle, &result));
        return result;
    }

    void set_mountdir() {
        EXPECT_SUCCESS(cryfs_mount_set_mountdir(handle, mountdir.path().native().c_str(), mountdir.path().native().size()));
    }

    void set_run_in_foreground(bool foreground = true) {
        EXPECT_SUCCESS(cryfs_mount_set_run_in_foreground(handle, foreground));
    }

    void mount() {
        EXPECT_SUCCESS(cryfs_mount(handle));
    }

    void unmount() {
        EXPECT_SUCCESS(cryfs_unmount(mountdir.path().native().c_str(), mountdir.path().native().size()));
    }

    void mount_filesystem() {
        set_mountdir();
        mount();
    }

    void create_and_mount_filesystem() {
        create_and_load_filesystem();
        mount_filesystem();
        std::this_thread::sleep_for(std::chrono::seconds(1)); // TODO Make cryfs_mount wait until mounted instead
    }

    void reload_and_mount_filesystem() {
        reinit_context();
        load_filesystem();
        mount_filesystem();
    }

    void create_file(const bf::path &filepath) {
        std::ofstream(filepath.c_str());
    }
};
const string Mount_Test::PASSWORD = "mypassword";
const string Mount_Test::NOTEXISTING_DIR = "/some/notexisting/dir";
const string Mount_Test::NOTEXISTING_LOGFILE = "/some/file/with/a/notexisting/parent/dir";
const string Mount_Test::INVALID_PATH = "pathname_with_some_invalid_characters_$% ä*.\\\"[]:;|=,";

TEST_F(Mount_Test, setup) {
    // Just test that the test setup works
    create_and_load_filesystem();
}

TEST_F(Mount_Test, get_cipher_1) {
    create_and_load_filesystem("aes-256-gcm");
    EXPECT_EQ("aes-256-gcm", get_ciphername(handle));
}

TEST_F(Mount_Test, get_cipher_2) {
    create_and_load_filesystem("twofish-256-gcm");
    EXPECT_EQ("twofish-256-gcm", get_ciphername(handle));
}

TEST_F(Mount_Test, set_mountdir_notexisting) {
    create_and_load_filesystem();
    EXPECT_EQ(cryfs_error_MOUNTDIR_DOESNT_EXIST, cryfs_mount_set_mountdir(handle, NOTEXISTING_DIR.c_str(), NOTEXISTING_DIR.size()));
}

TEST_F(Mount_Test, set_mountdir_invalid) {
    create_and_load_filesystem();
    EXPECT_EQ(cryfs_error_MOUNTDIR_DOESNT_EXIST, cryfs_mount_set_mountdir(handle, INVALID_PATH.c_str(), INVALID_PATH.size()));
}

TEST_F(Mount_Test, set_mountdir_valid) {
    create_and_load_filesystem();
    EXPECT_SUCCESS(cryfs_mount_set_mountdir(handle, mountdir.path().native().c_str(), mountdir.path().native().size()));
}

TEST_F(Mount_Test, set_run_in_foreground_true) {
    create_and_load_filesystem();
    EXPECT_SUCCESS(cryfs_mount_set_run_in_foreground(handle, true));
}

TEST_F(Mount_Test, set_run_in_foreground_false) {
    create_and_load_filesystem();
    EXPECT_SUCCESS(cryfs_mount_set_run_in_foreground(handle, false));
}

TEST_F(Mount_Test, set_logfile_notexisting) {
    create_and_load_filesystem();
    EXPECT_EQ(cryfs_error_INVALID_LOGFILE, cryfs_mount_set_logfile(handle, NOTEXISTING_LOGFILE.c_str(), NOTEXISTING_LOGFILE.size()));
}

TEST_F(Mount_Test, set_logfile_invalid) {
    create_and_load_filesystem();
    EXPECT_EQ(cryfs_error_INVALID_LOGFILE, cryfs_mount_set_logfile(handle, INVALID_PATH.c_str(), INVALID_PATH.size()));
}

TEST_F(Mount_Test, set_logfile_valid_notexisting) {
    create_and_load_filesystem();
    logfile.remove();
    EXPECT_SUCCESS(cryfs_mount_set_logfile(handle, logfile.path().native().c_str(), logfile.path().native().size()));
}

TEST_F(Mount_Test, set_logfile_valid_existing) {
    create_and_load_filesystem();
    EXPECT_SUCCESS(cryfs_mount_set_logfile(handle, logfile.path().native().c_str(), logfile.path().native().size()));
}

TEST_F(Mount_Test, set_unmount_idle) {
    create_and_load_filesystem();
    EXPECT_SUCCESS(cryfs_mount_set_unmount_idle(handle, 1000));
}

TEST_F(Mount_Test, set_fuse_argument) {
    const std::string ARGUMENT = "argument";
    create_and_load_filesystem();
    EXPECT_SUCCESS(cryfs_mount_add_fuse_argument(handle, ARGUMENT.c_str(), ARGUMENT.size()));
}

TEST_F(Mount_Test, set_fuse_argument_multiple) {
    const std::string ARGUMENT1 = "argument1";
    const std::string ARGUMENT2 = "another argument";
    const std::string ARGUMENT3 = "and a third one";
    create_and_load_filesystem();
    EXPECT_SUCCESS(cryfs_mount_add_fuse_argument(handle, ARGUMENT1.c_str(), ARGUMENT1.size()));
    EXPECT_SUCCESS(cryfs_mount_add_fuse_argument(handle, ARGUMENT2.c_str(), ARGUMENT2.size()));
    EXPECT_SUCCESS(cryfs_mount_add_fuse_argument(handle, ARGUMENT3.c_str(), ARGUMENT3.size()));
}

TEST_F(Mount_Test, mount_without_mountdir) {
    create_and_load_filesystem();
    EXPECT_EQ(cryfs_error_MOUNTDIR_NOT_SET, cryfs_mount(handle));
}

TEST_F(Mount_Test, mount) {
    create_and_load_filesystem();
    set_mountdir();
    EXPECT_SUCCESS(cryfs_mount(handle));
    std::this_thread::sleep_for(std::chrono::seconds(1)); // TODO Make cryfs_mount wait until mounted instead
    unmount(); // cleanup
}

TEST_F(Mount_Test, mount_in_background) {
    create_and_load_filesystem();
    set_mountdir();
    set_run_in_foreground(false);
    mount();
    // Test it is running in background. If it weren't, the call to mount() would be blocking and the test wouldn't continue.
    std::this_thread::sleep_for(std::chrono::seconds(1)); // TODO Make cryfs_mount wait until mounted instead
    unmount(); // cleanup
}

TEST_F(Mount_Test, mount_in_foreground) {
    create_and_load_filesystem();
    set_mountdir();
    set_run_in_foreground(true);

    UnmountAfterTimeout unmounter(mountdir.path());
    mount();
    EXPECT_TRUE(unmounter.timeoutPassed()); // Expect that we only get here once the unmount timeout passed.
}

TEST_F(Mount_Test, mountdir_is_correct) {
    const auto filepath = mountdir.path() / "myfile";
    create_and_mount_filesystem();
    EXPECT_FALSE(bf::exists(filepath));
    create_file(filepath);
    EXPECT_TRUE(bf::exists(filepath));
    std::this_thread::sleep_for(std::chrono::seconds(2)); // TODO This is needed, because the current implementation of daemonize() doesn't return to call the CryDevice destructor on unmount, so Cache won't be written back. Has to be fixed!
    unmount();
    EXPECT_FALSE(bf::exists(filepath));
    reload_and_mount_filesystem();
    std::this_thread::sleep_for(std::chrono::seconds(1)); // TODO Make cryfs_mount wait until mounted instead
    EXPECT_TRUE(bf::exists(filepath));
    unmount();
}

TEST_F(Mount_Test, basedir_is_correct) {
    create_and_mount_filesystem();
    auto numBasedirEntries = std::distance(bf::recursive_directory_iterator(basedir.path()), bf::recursive_directory_iterator());
    create_file(mountdir.path() / "myfile");
    unmount();
    auto newNumBasedirEntries = std::distance(bf::recursive_directory_iterator(basedir.path()), bf::recursive_directory_iterator());
    EXPECT_GT(newNumBasedirEntries, numBasedirEntries);
}

//TODO mount_logfilenotspecified_foreground_logstostderr
//TODO mount_logfilenotspecified_background_logstosyslog
//TODO mount_logfilespecified_foreground_logstofile
//TODO mount_logfilespecified_background_logstofile
//TODO Test fuse arguments are applied correctly (choose an easy-to-check argument. allowother for example?)
//TODO Test unmount_idle
