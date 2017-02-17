#include <gmock/gmock.h>
#include <cryfs/impl/config/CryConfig.h>
#include <cryfs/impl/config/CryConfigFile.h>
#include <cryfs/impl/filesystem/CryDevice.h>
#include <blockstore/implementations/ondisk/OnDiskBlockStore.h>
#include "testutils/Load_Test.h"
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
using testing::MatchesRegex;
using testing::HasSubstr;
using testing::Not;

class Mount_Test : public Load_Test {
public:
    cryfs_mount_handle *handle = nullptr;
    TempDir basedir;
    TempDir mountdir;
    TempFile logfile;
    TempFile _existing_file;
    static const string PASSWORD;
    static const string NOTEXISTING_DIR;
    static const string NOTEXISTING_LOGFILE;
    static const string INVALID_PATH;
    const string EXISTING_FILE = _existing_file.path().native();

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
        // TODO Run all these test cases twice (type parametrisation), once creating the file system and then using the load api, once using the create api.
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

    void set_unmount_idle_milliseconds(int milliseconds) {
        EXPECT_SUCCESS(cryfs_mount_set_unmount_idle_milliseconds(handle, milliseconds));
    }

    void set_logfile(const bf::path &path) {
        EXPECT_SUCCESS(cryfs_mount_set_logfile(handle, path.native().c_str(), path.native().size()));
    }

    void add_fuse_argument(const std::string &argument) {
        EXPECT_SUCCESS(cryfs_mount_add_fuse_argument(handle, argument.c_str(), argument.size()));
    }

    void mount() {
        EXPECT_SUCCESS(cryfs_mount(handle));
    }

    void unmount() {
        EXPECT_SUCCESS(cryfs_unmount(api, mountdir.path().native().c_str(), mountdir.path().native().size()));
    }

    void mount_filesystem() {
        set_mountdir();
        mount();
    }

    void create_and_mount_filesystem() {
        create_and_load_filesystem();
        mount_filesystem();
    }

    void reload_and_mount_filesystem() {
        reinit_context();
        load_filesystem();
        mount_filesystem();
    }

    void create_file(const bf::path &filepath) {
        std::ofstream(filepath.c_str());
    }

    std::chrono::milliseconds duration_timer(std::function<void()> func) {
        std::chrono::high_resolution_clock::time_point beginTime = std::chrono::high_resolution_clock::now();
        func();
        return std::chrono::duration_cast<std::chrono::milliseconds>(std::chrono::high_resolution_clock::now() - beginTime);
    }

    std::string captureStderr(std::function<void()> func) {
        testing::internal::CaptureStderr();
        func();
        return testing::internal::GetCapturedStderr();
    }

    std::string captureStdout(std::function<void()> func) {
        testing::internal::CaptureStdout();
        func();
        return testing::internal::GetCapturedStdout();
    }

    string loadFileContent(const bf::path &path) {
        std::ifstream file(path.c_str());
        return string(std::istreambuf_iterator<char>(file), std::istreambuf_iterator<char>());
    }

    void createFile(const bf::path &path) {
        std::ofstream(path.native().c_str());
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

TEST_F(Mount_Test, set_mountdir_is_file) {
    create_and_load_filesystem();
    EXPECT_EQ(cryfs_error_MOUNTDIR_INACCESSIBLE, cryfs_mount_set_mountdir(handle, EXISTING_FILE.c_str(), EXISTING_FILE.size()));
}

TEST_F(Mount_Test, set_mountdir_not_readable) {
    create_and_load_filesystem();
    chmod(mountdir.path().native().c_str(), S_IWUSR | S_IXUSR | S_IWGRP | S_IXGRP | S_IWOTH | S_IXOTH);
    EXPECT_EQ(cryfs_error_MOUNTDIR_INACCESSIBLE, cryfs_mount_set_mountdir(handle, mountdir.path().native().c_str(), mountdir.path().native().size()));
}

TEST_F(Mount_Test, set_mountdir_not_writeble) {
    create_and_load_filesystem();
    chmod(mountdir.path().native().c_str(), S_IRUSR | S_IXUSR | S_IRGRP | S_IXGRP | S_IROTH | S_IXOTH);
    EXPECT_EQ(cryfs_error_MOUNTDIR_INACCESSIBLE, cryfs_mount_set_mountdir(handle, mountdir.path().native().c_str(), mountdir.path().native().size()));
}

TEST_F(Mount_Test, set_mountdir_not_enterable) {
    create_and_load_filesystem();
    chmod(mountdir.path().native().c_str(), S_IRUSR | S_IWUSR | S_IRGRP | S_IWGRP | S_IROTH | S_IWOTH);
    EXPECT_EQ(cryfs_error_MOUNTDIR_INACCESSIBLE, cryfs_mount_set_mountdir(handle, mountdir.path().native().c_str(), mountdir.path().native().size()));
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

TEST_F(Mount_Test, set_logfile_not_writable) {
    create_and_load_filesystem();
    chmod(logfile.path().native().c_str(), S_IRUSR | S_IXUSR | S_IRGRP | S_IXGRP | S_IROTH | S_IXOTH);
    EXPECT_EQ(cryfs_error_LOGFILE_NOT_WRITABLE, cryfs_mount_set_logfile(handle, logfile.path().native().c_str(), logfile.path().native().size()));
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

TEST_F(Mount_Test, set_unmount_idle_milliseconds) {
    create_and_load_filesystem();
    EXPECT_SUCCESS(cryfs_mount_set_unmount_idle_milliseconds(handle, 1000));
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

    unmount(); // cleanup
}

TEST_F(Mount_Test, mount_in_background) {
    create_and_load_filesystem();
    set_mountdir();
    set_run_in_foreground(false);
    mount();
    // Test it is running in background. If it weren't, the call to mount() would be blocking and the test wouldn't continue.
    unmount(); // cleanup
}

TEST_F(Mount_Test, mount_in_foreground) {
    create_and_load_filesystem();
    set_mountdir();
    set_run_in_foreground(true);

    UnmountAfterTimeout unmounter(api, mountdir.path(), boost::chrono::milliseconds(2000));
    mount();
    EXPECT_TRUE(unmounter.timeoutPassed()); // Expect that we only get here once the unmount timeout passed.
}

TEST_F(Mount_Test, mountdir_is_correct) {
    const auto filepath = mountdir.path() / "myfile";
    create_and_mount_filesystem();
    EXPECT_FALSE(bf::exists(filepath));
    create_file(filepath);
    EXPECT_TRUE(bf::exists(filepath));
    unmount();
    EXPECT_FALSE(bf::exists(filepath));
    reload_and_mount_filesystem();
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

TEST_F(Mount_Test, unmount_idle_zero) {
    create_and_load_filesystem();
    set_mountdir();
    set_run_in_foreground(true);
    set_unmount_idle_milliseconds(0);
    auto duration = duration_timer([this] {
        mount();
    });
    EXPECT_LT(duration, std::chrono::milliseconds(1000));
}

TEST_F(Mount_Test, unmount_idle_small) {
    create_and_load_filesystem();
    set_mountdir();
    set_run_in_foreground(true);
    set_unmount_idle_milliseconds(1000);
    auto duration = duration_timer([this] {
        mount();
    });
    EXPECT_GT(duration, std::chrono::milliseconds(500));
    EXPECT_LT(duration, std::chrono::milliseconds(1500));
}

TEST_F(Mount_Test, unmount_idle_large) {
    create_and_load_filesystem();
    set_mountdir();
    set_run_in_foreground(true);
    set_unmount_idle_milliseconds(5000);
    auto duration = duration_timer([this] {
        mount();
    });
    EXPECT_GT(duration, std::chrono::milliseconds(4500));
    EXPECT_LT(duration, std::chrono::milliseconds(5500));
}

TEST_F(Mount_Test, mount_logfilenotspecified_foreground_logstostderr) {
    create_and_load_filesystem();
    set_mountdir();
    set_run_in_foreground(true);
    set_unmount_idle_milliseconds(0);

    string stderr = captureStderr([this] {
        mount();
    });
    EXPECT_THAT(stderr, MatchesRegex(".*Filesystem started.*Filesystem stopped.*"));
}

TEST_F(Mount_Test, mount_logfilenotspecified_foreground_doesntlogstostdout) {
    create_and_load_filesystem();
    set_mountdir();
    set_run_in_foreground(true);
    set_unmount_idle_milliseconds(0);

    string stdout = captureStdout([this] {
        mount();
    });
    EXPECT_THAT(stdout, Not(HasSubstr("Filesystem started")));
    EXPECT_THAT(stdout, Not(HasSubstr("Filesystem stopped")));
}

/* TODO Don't know how to test this, because syslog is hard to access platform independently.
 *      http://stackoverflow.com/questions/37867356/capture-syslog-for-test-cases/37867750#37867750
 *      Maybe use DI to insert logging into all classes?
 *
TEST_F(Mount_Test, mount_logfilenotspecified_foreground_doesntlogstosyslog) {
    create_and_load_filesystem();
    set_mountdir();
    set_run_in_foreground(true);
    set_unmount_idle_milliseconds(0);

    string syslog = captureSyslog([this] {
        mount();
    });
    EXPECT_THAT(syslog, Not(HasSubstr("Filesystem started")));
    EXPECT_THAT(syslog, Not(HasSubstr("Filesystem stopped")));
}*/

TEST_F(Mount_Test, mount_logfilespecified_foreground_doesntlogstostderr) {
    create_and_load_filesystem();
    set_mountdir();
    set_run_in_foreground(true);
    set_unmount_idle_milliseconds(0);
    TempFile file;
    set_logfile(file.path());

    string stderr = captureStderr([this] {
        mount();
    });
    EXPECT_THAT(stderr, Not(HasSubstr("Filesystem started")));
    EXPECT_THAT(stderr, Not(HasSubstr("Filesystem stopped")));
}

TEST_F(Mount_Test, mount_logfilespecified_foreground_doesntlogstostdout) {
    create_and_load_filesystem();
    set_mountdir();
    set_run_in_foreground(true);
    set_unmount_idle_milliseconds(0);
    TempFile file;
    set_logfile(file.path());

    string stdout = captureStdout([this] {
        mount();
    });
    EXPECT_THAT(stdout, Not(HasSubstr("Filesystem started")));
    EXPECT_THAT(stdout, Not(HasSubstr("Filesystem stopped")));
}

/* TODO Don't know how to test this, because syslog is hard to access platform independently.
 *      http://stackoverflow.com/questions/37867356/capture-syslog-for-test-cases/37867750#37867750
 *      Maybe use DI to insert logging into all classes?
 *
TEST_F(Mount_Test, mount_logfilespecified_foreground_doesntlogstosyslog) {
    create_and_load_filesystem();
    set_mountdir();
    set_run_in_foreground(true);
    set_unmount_idle_milliseconds(0);
    TempFile file;
    set_logfile(file.path());

    string syslog = captureSyslog([this] {
        mount();
    });
    EXPECT_THAT(syslog, Not(HasSubstr("Filesystem started")));
    EXPECT_THAT(syslog, Not(HasSubstr("Filesystem stopped")));
}*/


TEST_F(Mount_Test, mount_logfilespecified_foreground_logstofile) {
    create_and_load_filesystem();
    set_mountdir();
    set_run_in_foreground(true);
    set_unmount_idle_milliseconds(0);
    TempFile file;
    set_logfile(file.path());

    mount();

    string filecontent = loadFileContent(file.path());
    EXPECT_THAT(filecontent, MatchesRegex(".*Filesystem started.*Filesystem stopped.*"));
}

/* TODO Don't know how to test this, because syslog is hard to access platform independently.
 *      http://stackoverflow.com/questions/37867356/capture-syslog-for-test-cases/37867750#37867750
 *      Maybe use DI to insert logging into all classes?
 *
TEST_F(Mount_Test, mount_logfilenotspecified_background_logstosyslog) {
    create_and_load_filesystem();
    set_mountdir();
    set_run_in_foreground(false);

    string syslog = captureSyslog([this] {
        mount();
        unmount();
    });

    EXPECT_THAT(syslog, MatchesRegex(".*Filesystem started.*Filesystem stopped.*"));
}*/

TEST_F(Mount_Test, mount_logfilenotspecified_background_doesntlogstostdout) {
    create_and_load_filesystem();
    set_mountdir();
    set_run_in_foreground(false);

    string stdout = captureStdout([this] {
        mount();
        unmount();
    });
    EXPECT_THAT(stdout, Not(HasSubstr("Filesystem started")));
    EXPECT_THAT(stdout, Not(HasSubstr("Filesystem stopped")));
}

TEST_F(Mount_Test, mount_logfilenotspecified_background_doesntlogstostderr) {
    create_and_load_filesystem();
    set_mountdir();
    set_run_in_foreground(false);

    string stderr = captureStderr([this] {
        mount();
        unmount();
    });
    EXPECT_THAT(stderr, Not(HasSubstr("Filesystem started")));
    EXPECT_THAT(stderr, Not(HasSubstr("Filesystem stopped")));
}

TEST_F(Mount_Test, mount_logfilespecified_background_doesntlogstostderr) {
    create_and_load_filesystem();
    set_mountdir();
    set_run_in_foreground(false);
    TempFile file;
    set_logfile(file.path());

    string stderr = captureStderr([this] {
        mount();
        unmount();
    });
    EXPECT_THAT(stderr, Not(HasSubstr("Filesystem started")));
    EXPECT_THAT(stderr, Not(HasSubstr("Filesystem stopped")));
}

TEST_F(Mount_Test, mount_logfilespecified_background_doesntlogstostdout) {
    create_and_load_filesystem();
    set_mountdir();
    set_run_in_foreground(false);
    TempFile file;
    set_logfile(file.path());

    string stdout = captureStdout([this] {
        mount();
        unmount();
    });
    EXPECT_THAT(stdout, Not(HasSubstr("Filesystem started")));
    EXPECT_THAT(stdout, Not(HasSubstr("Filesystem stopped")));
}

/* TODO Don't know how to test this, because syslog is hard to access platform independently.
 *      http://stackoverflow.com/questions/37867356/capture-syslog-for-test-cases/37867750#37867750
 *      Maybe use DI to insert logging into all classes?
 *
TEST_F(Mount_Test, mount_logfilespecified_background_doesntlogstosyslog) {
    create_and_load_filesystem();
    set_mountdir();
    set_run_in_foreground(false);
    TempFile file;
    set_logfile(file.path());

    string syslog = captureSyslog([this] {
        mount();
        unmount();
    });
    EXPECT_THAT(syslog, Not(HasSubstr("Filesystem started")));
    EXPECT_THAT(syslog, Not(HasSubstr("Filesystem stopped")));
}*/

TEST_F(Mount_Test, mount_logfilespecified_background_logstofile) {
    create_and_load_filesystem();
    set_mountdir();
    set_run_in_foreground(false);
    TempFile file;
    set_logfile(file.path());

    mount();
    unmount();
    std::this_thread::sleep_for(std::chrono::milliseconds(500)); // give cryfs some time to exit and flush the log

    string filecontent = loadFileContent(file.path());

    EXPECT_THAT(filecontent, MatchesRegex(".*Filesystem started.*Filesystem stopped.*"));
}

TEST_F(Mount_Test, mount_fusearguments) {
    create_and_load_filesystem();
    set_mountdir();
    createFile(mountdir.path() / "myfile");

    // Expect mounting to fail because mountdir is not empty
    EXPECT_EQ(cryfs_error_UNKNOWN_ERROR, cryfs_mount(handle));

    add_fuse_argument("-o");
    add_fuse_argument("nonempty");

    // Now expect mounting to succeed
    EXPECT_SUCCESS(cryfs_mount(handle));
    unmount();
}
