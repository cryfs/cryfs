#pragma once
#ifndef MESSMER_CRYFS_TEST_CLI_TESTUTILS_CLITEST_H
#define MESSMER_CRYFS_TEST_CLI_TESTUTILS_CLITEST_H

#include <gtest/gtest.h>
#include <gmock/gmock.h>
#include <cpp-utils/tempfile/TempDir.h>
#include <cpp-utils/tempfile/TempFile.h>
#include <cryfs-cli/Cli.h>
#include <cryfs-cli/VersionChecker.h>
#include <cpp-utils/logging/logging.h>
#include <cpp-utils/process/subprocess.h>
#include <cpp-utils/network/FakeHttpClient.h>
#include "../../cryfs/testutils/MockConsole.h"
#include "../../cryfs/testutils/TestWithFakeHomeDirectory.h"
#include <cryfs/ErrorCodes.h>
#include <cpp-utils/testutils/CaptureStderrRAII.h>

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
        return std::move(httpClient);
    }

    int run(std::vector<const char*> args) {
        std::vector<const char*> _args;
        _args.reserve(args.size()+1);
        _args.push_back("cryfs");
        for (const char *arg : args) {
            _args.push_back(arg);
        }
        auto &keyGenerator = cpputils::Random::PseudoRandom();
        ON_CALL(*console, askPassword(testing::StrEq("Password: "))).WillByDefault(testing::Return("pass"));
        ON_CALL(*console, askPassword(testing::StrEq("Confirm Password: "))).WillByDefault(testing::Return("pass"));
        // Run Cryfs
        return cryfs::Cli(keyGenerator, cpputils::SCrypt::TestSettings, console).main(_args.size(), _args.data(), _httpClient());
    }

    void EXPECT_EXIT_WITH_HELP_MESSAGE(std::vector<const char*> args, const std::string &message, cryfs::ErrorCode errorCode) {
        EXPECT_RUN_ERROR(args, (".*Usage:.*"+message).c_str(), errorCode);
    }

    void EXPECT_RUN_ERROR(std::vector<const char*> args, const char* message, cryfs::ErrorCode errorCode) {
        cpputils::CaptureStderrRAII capturedStderr;
        int exit_code = run(args);
        capturedStderr.EXPECT_MATCHES(string(".*") + message + ".*");
        EXPECT_EQ(exitCode(errorCode), exit_code);
    }

    void EXPECT_RUN_SUCCESS(std::vector<const char*> args, const boost::filesystem::path &mountDir) {
        //TODO Make this work when run in background
        ASSERT(std::find(args.begin(), args.end(), string("-f")) != args.end(), "Currently only works if run in foreground");
        std::thread unmountThread([&mountDir] {
            int returncode = -1;
            while (returncode != 0) {
#ifdef __APPLE__
                returncode = cpputils::Subprocess::call(std::string("umount ") + mountDir.string().c_str() + " 2>/dev/null").exitcode;
#else
                returncode = cpputils::Subprocess::call(std::string("fusermount -u ") + mountDir.string().c_str() + " 2>/dev/null").exitcode;
#endif
                //std::this_thread::sleep_for(std::chrono::milliseconds(50)); // TODO Is this the test case duration? Does a shorter interval make the test case faster?
            }
        });
        testing::internal::CaptureStdout();
        run(args);
        unmountThread.join();
        EXPECT_THAT(testing::internal::GetCapturedStdout(), testing::MatchesRegex(".*Mounting filesystem.*"));
    }
};

#endif
