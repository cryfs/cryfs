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
#include "../../cryfs/testutils/MockConsole.h"
#include "../../cryfs/testutils/TestWithFakeHomeDirectory.h"
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
        capturedStderr.EXPECT_MATCHES(message);
        EXPECT_EQ(exitCode(errorCode), exit_code);
    }

    void EXPECT_RUN_SUCCESS(std::vector<const char*> args, const boost::filesystem::path &mountDir) {
        //TODO Make this work when run in background
        ASSERT(std::find(args.begin(), args.end(), string("-f")) != args.end(), "Currently only works if run in foreground");
		bool unmount_success = false;
        std::thread unmountThread([&mountDir, &unmount_success] {
			auto start = std::chrono::steady_clock::now();
            int returncode = -1;
            while (returncode != 0) {
				if (std::chrono::steady_clock::now() - start > std::chrono::seconds(10)) {
					return; // keep unmount_success = false
				}
#if defined(__APPLE__)
                returncode = cpputils::Subprocess::call(std::string("umount ") + mountDir.string().c_str() + " 2>/dev/null").exitcode;
#elif defined(_MSC_VER)
				// Somehow this sleeping is needed to not deadlock. Race condition in mounting/unmounting?
				std::this_thread::sleep_for(std::chrono::milliseconds(50));
				std::wstring mountDir_ = std::wstring_convert<std::codecvt_utf8_utf16<wchar_t>>().from_bytes(mountDir.string());
				BOOL success = DokanRemoveMountPoint(mountDir_.c_str());
				returncode = success ? 0 : -1;
#else
                returncode = cpputils::Subprocess::call(std::string("fusermount -u ") + mountDir.string().c_str() + " 2>/dev/null").exitcode;
#endif
            }
			unmount_success = true;
        });
        testing::internal::CaptureStdout();
        run(args);
		std::string _stdout = testing::internal::GetCapturedStdout();
        unmountThread.join();

		EXPECT_TRUE(unmount_success);

		// For some reason, the following doesn't seem to work in MSVC. Possibly because of the multiline string?
        // EXPECT_THAT(testing::internal::GetCapturedStdout(), testing::MatchesRegex(".*Mounting filesystem.*"));
		EXPECT_TRUE(std::regex_search(_stdout, std::regex(".*Mounting filesystem.*")));
    }
};

#endif
