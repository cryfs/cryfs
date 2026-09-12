#pragma once
#ifndef MESSMER_CRYFS_TEST_CLI_TESTUTILS_CLITEST_H
#define MESSMER_CRYFS_TEST_CLI_TESTUTILS_CLITEST_H

#if defined(_MSC_VER)
#include <codecvt>
#include <dokan/dokan.h>
#endif

#include <gtest/gtest.h>
#include <gmock/gmock.h>
#include <cpp-utils/tempfile/TempDir.h>
#include <cpp-utils/tempfile/TempFile.h>
#include <cryfs-cli/Cli.h>
#include <cryfs-cli/VersionChecker.h>
#include <cpp-utils/logging/logging.h>
#include <cpp-utils/process/subprocess.h>
#include <cpp-utils/network/FakeHttpClient.h>
#include <cpp-utils/lock/ConditionBarrier.h>
#include "../../cryfs/impl/testutils/MockConsole.h"
#include "../../cryfs/impl/testutils/TestWithFakeHomeDirectory.h"
#include <fspp/fuse/Fuse.h>
#include <cryfs/impl/ErrorCodes.h>
#include <cpp-utils/testutils/CaptureStderrRAII.h>
#include <regex>
#include <string>
#include <fstream>
#include <thread>
#include <chrono>
#include <cstdlib>

// EXPLORATION ONLY: trace the harness steps to a file, because gtest captures stdout/stderr here.
inline void harness_trace(const std::string& msg) {
    const char* file = std::getenv("CRYFS_TEST_TRACE_FILE");
    if (file != nullptr) {
        std::ofstream f(file, std::ios::app);
        f << msg << std::endl;
    }
}

#if defined(_MSC_VER)
namespace {
// CryFS on Windows mounts to a drive letter, not into a directory (see Fuse::_run), so a test that
// expects a mount to succeed needs a drive letter nothing else is using.
inline boost::filesystem::path find_free_drive_letter() {
    const DWORD used = GetLogicalDrives();
    for (char letter = 'Z'; letter >= 'D'; --letter) {
        if (0 == (used & (1u << (letter - 'A')))) {
            return boost::filesystem::path(std::string(1, letter) + ":");
        }
    }
    throw std::runtime_error("Didn't find a free drive letter to mount the test file system to");
}
}
#endif

class CliTest : public ::testing::Test, TestWithFakeHomeDirectory {
public:
    CliTest(): _basedir(), _mountdir(), basedir(_basedir.path()), mountdir(_mountdir.path()),
#if defined(_MSC_VER)
        mountpoint(find_free_drive_letter()),
#else
        mountpoint(_mountdir.path()),
#endif
        logfile(), configfile(false), console(std::make_shared<MockConsole>()) {}

    cpputils::TempDir _basedir;
    cpputils::TempDir _mountdir;
    boost::filesystem::path basedir;
    // A directory to test the checks CryFS does on its mount directory with. On Linux and macOS
    // the tests that expect the mount to succeed also mount into it. On Windows, CryFS can only
    // mount to a drive letter (see Fuse::_run), so there they mount to `mountpoint` instead and
    // the tests that need the mount directory to be a directory are skipped.
    boost::filesystem::path mountdir;
    // Where a test that expects the mount to succeed mounts to: `mountdir` on Linux and macOS, a
    // free drive letter on Windows.
    boost::filesystem::path mountpoint;
    cpputils::TempFile logfile;
    cpputils::TempFile configfile;
    std::shared_ptr<MockConsole> console;

    // A path inside the mounted file system. On Windows `mountpoint` is a bare drive letter, and
    // "X:myfile" would be relative to that drive's current directory rather than to its root.
    boost::filesystem::path in_mountpoint(const std::string& name) const {
#if defined(_MSC_VER)
        return boost::filesystem::path(mountpoint.string() + "\\") / name;
#else
        return mountpoint / name;
#endif
    }

    cpputils::unique_ref<cpputils::HttpClient> _httpClient() {
        cpputils::unique_ref<cpputils::FakeHttpClient> httpClient = cpputils::make_unique_ref<cpputils::FakeHttpClient>();
        httpClient->addWebsite("https://www.cryfs.org/version_info.json", "{\"version_info\":{\"current\":\"0.8.5\"}}");
        return httpClient;
    }

    int run(const std::vector<std::string>& args, std::function<void()> onMounted) {
        std::vector<const char*> _args;
        _args.reserve(args.size() + 1);
        _args.emplace_back("cryfs");
        for (const std::string& arg : args) {
            _args.emplace_back(arg.c_str());
        }
        auto *keyGenerator = cpputils::Random::Csprng();
        ON_CALL(*console, askPassword(testing::StrEq("Password: "))).WillByDefault(testing::Return("pass"));
        ON_CALL(*console, askPassword(testing::StrEq("Confirm Password: "))).WillByDefault(testing::Return("pass"));
        // Run Cryfs
        return cryfs_cli::Cli(keyGenerator, cpputils::SCrypt::TestSettings, console).main(
            _args.size(),
            _args.data(),
            #ifdef CRYFS_UPDATE_CHECKS
            _httpClient(),
            #endif
            std::move(onMounted)
        );
    }

    void EXPECT_EXIT_WITH_HELP_MESSAGE(const std::vector<std::string>& args, const std::string &message, cryfs::ErrorCode errorCode) {
        EXPECT_RUN_ERROR(args, "Usage:[^\\x00]*"+message, errorCode);
    }

    enum class UnmountAfterwards { Yes, No };

    // `mountDir` is where the file system gets mounted if the run gets that far. It is only needed
    // when `onMounted` accesses the mounted file system, see run_filesystem().
    void EXPECT_RUN_ERROR(const std::vector<std::string>& args, const std::string& message, cryfs::ErrorCode errorCode, std::function<void ()> onMounted = [] {}, const boost::optional<boost::filesystem::path> &mountDir = boost::none) {
        const FilesystemOutput filesystem_output = run_filesystem(args, mountDir, UnmountAfterwards::No, std::move(onMounted));

        EXPECT_EQ(exitCode(errorCode), filesystem_output.exit_code);
        if (!std::regex_search(filesystem_output.stderr_, std::regex(message))) {
            std::cerr << filesystem_output.stderr_ << std::endl;
            EXPECT_TRUE(false);
        }
    }

    // `mountDir` is where the file system gets mounted. It gets unmounted after `onMounted` ran,
    // unless the test unmounts it itself and says so with UnmountAfterwards::No.
    void EXPECT_RUN_SUCCESS(const std::vector<std::string>& args, const boost::filesystem::path &mountDir, std::function<void ()> onMounted = [] {}, UnmountAfterwards unmountAfterwards = UnmountAfterwards::Yes) {
        //TODO Make this work when run in background
        ASSERT(std::find(args.begin(), args.end(), string("-f")) != args.end(), "Currently only works if run in foreground");

        bool successfully_mounted = false;

        const FilesystemOutput filesystem_output = run_filesystem(args, mountDir, unmountAfterwards, [&] {
            successfully_mounted = true;
            onMounted();
        });

        EXPECT_EQ(0, filesystem_output.exit_code);
        if (!std::regex_search(filesystem_output.stdout_, std::regex("Mounting filesystem"))) {
          std::cerr << "STDOUT:\n" << filesystem_output.stdout_ << "STDERR:\n" << filesystem_output.stderr_ << std::endl;
          EXPECT_TRUE(false) << "Filesystem did not output the 'Mounting filesystem' message, probably wasn't successfully mounted.";
        }

        if (!successfully_mounted) {
            EXPECT_TRUE(false) << "Filesystem did not call onMounted callback, probably wasn't successfully mounted.";
        }
    }

    struct FilesystemOutput final {
        int exit_code;
        std::string stdout_;
        std::string stderr_;
    };

    static void _unmount(const boost::filesystem::path &mountDir) {
        fspp::fuse::Fuse::unmount(mountDir, true);
    }

    // Waits until the mounted file system shows up at `mountDir`. The onMounted callback that
    // releases the barrier below is called from Fuse::init(). libfuse calls that once the mount is
    // live, so on Linux and macOS this returns right away. Dokany calls it while it is still
    // setting the mount up: for a moment after it the drive letter isn't there yet, and both
    // accessing the file system and DokanRemoveMountPoint() fail until it is.
    static void _waitUntilMounted(const boost::filesystem::path &mountDir) {
#if defined(_MSC_VER)
        // `mountDir` is a bare drive letter, and "Z:" alone is relative to that drive's current directory
        const boost::filesystem::path root = mountDir.string() + "\\";
#else
        const boost::filesystem::path &root = mountDir;
#endif
        const auto deadline = std::chrono::steady_clock::now() + std::chrono::seconds(30);
        while (!boost::filesystem::exists(root)) {
            if (std::chrono::steady_clock::now() > deadline) {
                throw std::runtime_error("Timeout waiting for the file system to show up at " + mountDir.string());
            }
            std::this_thread::sleep_for(std::chrono::milliseconds(10));
        }
    }

    // `mountDir` is where the file system gets mounted, if it does. When it is given, the file
    // system is unmounted after `onMounted` ran, unless `unmountAfterwards` says the test does that
    // itself.
    FilesystemOutput run_filesystem(const std::vector<std::string>& args, boost::optional<boost::filesystem::path> mountDir, UnmountAfterwards unmountAfterwards, std::function<void()> onMounted) {
        testing::internal::CaptureStdout();
        testing::internal::CaptureStderr();
        try {
            return _run_filesystem_with_captured_output(args, std::move(mountDir), unmountAfterwards, std::move(onMounted));
        } catch (...) {
            // The exception is reported by gtest once it propagates out of the test body, but
            // gtest prints that report to stdout, which is still being captured here. Stop
            // capturing first, and show what the file system printed, because that is where
            // the reason usually is.
            std::cerr << "Running the file system threw an exception.\nSTDOUT:\n" << testing::internal::GetCapturedStdout()
                      << "STDERR:\n" << testing::internal::GetCapturedStderr() << std::endl;
            throw;
        }
    }

    FilesystemOutput _run_filesystem_with_captured_output(const std::vector<std::string>& args, boost::optional<boost::filesystem::path> mountDir, UnmountAfterwards unmountAfterwards, std::function<void()> onMounted) {
        bool exited = false;
        cpputils::ConditionBarrier isMountedOrFailedBarrier;

        std::future<int> exit_code = std::async(std::launch::async, [&] {
            // Release the barrier however run() ends, also when it throws. If it fails before
            // mounting, this releases the barrier the mount would have released. If it mounted,
            // this releases it a second time, which doesn't hurt. Without the release on an
            // exception, the thread below would wait for the whole timeout and then abort the
            // test binary, and the exception would never be reported.
            struct ReleaseBarrier final {
                bool* exited;
                cpputils::ConditionBarrier* barrier;
                ~ReleaseBarrier() {
                    *exited = true;
                    barrier->release();
                }
            } releaseBarrier{&exited, &isMountedOrFailedBarrier};
            harness_trace("run(): starting Cli::main");
            const int code = run(args, [&] { harness_trace("onMounted callback from Fuse::init"); isMountedOrFailedBarrier.release(); });
            harness_trace("run(): Cli::main returned " + std::to_string(code));
            return code;
        });

        std::future<bool> on_mounted_success = std::async(std::launch::async, [&] {
            isMountedOrFailedBarrier.wait();
            if (exited) {
              // file system already exited on its own, this indicates an error. It should have stayed mounted.
              // while the exit_code from run() will signal an error in this case, we didn't encounter another
              // error in the onMounted future, so return true here.
              return true;
            }
            // now we know the filesystem stayed online, so we can call the onMounted callback
            const bool unmount = mountDir.is_initialized() && unmountAfterwards == UnmountAfterwards::Yes;
            try {
              if (mountDir.is_initialized()) {
                harness_trace("on_mounted thread: waiting for " + mountDir->string());
                _waitUntilMounted(*mountDir);
              }
              harness_trace("on_mounted thread: calling the test's onMounted");
              onMounted();
              harness_trace("on_mounted thread: onMounted returned");
            } catch (...) {
              // Unmount anyway if we can, otherwise Cli::main() never returns and instead of
              // reporting this exception, the test would wait for the timeout below.
              if (unmount) {
                try {
                  _unmount(*mountDir);
                } catch (...) {
                  // the original exception is the one worth reporting
                }
              }
              throw;
            }
            // and unmount it afterwards
            if (unmount) {
              harness_trace("on_mounted thread: unmounting " + mountDir->string());
              try {
                _unmount(*mountDir);
              } catch (const std::exception& e) {
                harness_trace(std::string("on_mounted thread: unmount threw: ") + e.what());
                throw;
              }
              harness_trace("on_mounted thread: unmount returned");
            }
            return true;
        });

        if(std::future_status::ready != on_mounted_success.wait_for(std::chrono::seconds(1000))) {
            testing::internal::GetCapturedStdout(); // stop capturing stdout
            testing::internal::GetCapturedStderr(); // stop capturing stderr

            std::cerr << "onMounted thread (e.g. used for unmount) didn't finish" << std::endl;
            // The std::future destructor of a future created with std::async blocks until the future is ready.
            // so, instead of causing a deadlock, rather abort
            exit(EXIT_FAILURE);
        }
        EXPECT_TRUE(on_mounted_success.get()); // this also re-throws any potential exceptions

        if(std::future_status::ready != exit_code.wait_for(std::chrono::seconds(1000))) {
            testing::internal::GetCapturedStdout(); // stop capturing stdout
            testing::internal::GetCapturedStderr(); // stop capturing stderr

            std::cerr << "Filesystem thread didn't finish" << std::endl;
            // The std::future destructor of a future created with std::async blocks until the future is ready.
            // so, instead of causing a deadlock, rather abort
            exit(EXIT_FAILURE);
        }

        return {
          exit_code.get(), // this also re-throws any potential exceptions
          testing::internal::GetCapturedStdout(),
          testing::internal::GetCapturedStderr()
        };
    }
};

#endif
