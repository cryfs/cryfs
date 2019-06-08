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
#include "../../cryfs/testutils/MockConsole.h"
#include "../../cryfs/testutils/TestWithFakeHomeDirectory.h"
#include <fspp/fuse/Fuse.h>
#include <cryfs/ErrorCodes.h>
#include <cpp-utils/testutils/CaptureStderrRAII.h>
#include <regex>
#include <string>

class CliTest : public ::testing::Test, TestWithFakeHomeDirectory {
public:
    CliTest(): _basedir(), _mountdir(), basedir(_basedir.path()), mountdir(_mountdir.path()), logfile(), configfile(false), console(std::make_shared<MockConsole>()) {}

    cpputils::TempDir _basedir;
    cpputils::TempDir _mountdir;
    boost::filesystem::path basedir;
    boost::filesystem::path mountdir;
    cpputils::TempFile logfile;
    cpputils::TempFile configfile;
    std::shared_ptr<MockConsole> console;

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
        auto &keyGenerator = cpputils::Random::PseudoRandom();
        ON_CALL(*console, askPassword(testing::StrEq("Password: "))).WillByDefault(testing::Return("pass"));
        ON_CALL(*console, askPassword(testing::StrEq("Confirm Password: "))).WillByDefault(testing::Return("pass"));
        // Run Cryfs
        return cryfs_cli::Cli(keyGenerator, cpputils::SCrypt::TestSettings, console).main(_args.size(), _args.data(), _httpClient(), std::move(onMounted));
    }

    void EXPECT_EXIT_WITH_HELP_MESSAGE(const std::vector<std::string>& args, const std::string &message, cryfs::ErrorCode errorCode) {
        EXPECT_RUN_ERROR(args, "Usage:[^\\x00]*"+message, errorCode);
    }

    void EXPECT_RUN_ERROR(const std::vector<std::string>& args, const std::string& message, cryfs::ErrorCode errorCode, std::function<void ()> onMounted = [] {}) {
        FilesystemOutput filesystem_output = run_filesystem(args, boost::none, std::move(onMounted));

        EXPECT_EQ(exitCode(errorCode), filesystem_output.exit_code);
        if (!std::regex_search(filesystem_output.stderr_, std::regex(message))) {
            std::cerr << filesystem_output.stderr_ << std::endl;
            EXPECT_TRUE(false);
        }
    }

    void EXPECT_RUN_SUCCESS(const std::vector<std::string>& args, const boost::optional<boost::filesystem::path> &mountDir, std::function<void ()> onMounted = [] {}) {
        //TODO Make this work when run in background
        ASSERT(std::find(args.begin(), args.end(), string("-f")) != args.end(), "Currently only works if run in foreground");

        FilesystemOutput filesystem_output = run_filesystem(args, mountDir, std::move(onMounted));

        EXPECT_EQ(0, filesystem_output.exit_code);
        if (!std::regex_search(filesystem_output.stdout_, std::regex("Mounting filesystem"))) {
          std::cerr << filesystem_output.stdout_ << std::endl;
          EXPECT_TRUE(false);
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

    FilesystemOutput run_filesystem(const std::vector<std::string>& args, boost::optional<boost::filesystem::path> mountDirForUnmounting, std::function<void()> onMounted) {
        testing::internal::CaptureStdout();
        testing::internal::CaptureStderr();

        bool exited = false;
        cpputils::ConditionBarrier isMountedOrFailedBarrier;

        std::future<int> exit_code = std::async(std::launch::async, [&] {
            int exit_code = run(args, [&] { isMountedOrFailedBarrier.release(); });
            // just in case it fails, we also want to release the barrier.
            // if it succeeds, this will release it a second time, which doesn't hurt.
            exited = true;
            isMountedOrFailedBarrier.release();
            return exit_code;
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
            onMounted();
            // and unmount it afterwards
            if (mountDirForUnmounting.is_initialized()) {
              _unmount(*mountDirForUnmounting);
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
