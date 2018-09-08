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

    int run(const std::vector<std::string>& args) {
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
        return cryfs::Cli(keyGenerator, cpputils::SCrypt::TestSettings, console).main(_args.size(), _args.data(), _httpClient());
    }

    void EXPECT_EXIT_WITH_HELP_MESSAGE(const std::vector<std::string>& args, const std::string &message, cryfs::ErrorCode errorCode) {
        EXPECT_RUN_ERROR(args, "Usage:[^\\x00]*"+message, errorCode);
    }

    void EXPECT_RUN_ERROR(const std::vector<std::string>& args, const std::string& message, cryfs::ErrorCode errorCode) {
        FilesystemOutput filesystem_output = _run_filesystem(args, boost::none);

        EXPECT_EQ(exitCode(errorCode), filesystem_output.exit_code);
        EXPECT_TRUE(std::regex_search(filesystem_output.stderr_, std::regex(message)));
    }

    void EXPECT_RUN_SUCCESS(const std::vector<std::string>& args, const boost::filesystem::path &mountDir) {
        //TODO Make this work when run in background
        ASSERT(std::find(args.begin(), args.end(), string("-f")) != args.end(), "Currently only works if run in foreground");

		FilesystemOutput filesystem_output = _run_filesystem(args, mountDir);

		EXPECT_EQ(0, filesystem_output.exit_code);
		EXPECT_TRUE(std::regex_search(filesystem_output.stdout_, std::regex("Mounting filesystem")));
    }

    struct FilesystemOutput final {
        int exit_code;
        std::string stdout_;
        std::string stderr_;
    };

    FilesystemOutput _run_filesystem(const std::vector<std::string>& args, const boost::optional<boost::filesystem::path>& mountDirForUnmounting) {
		testing::internal::CaptureStdout();
		testing::internal::CaptureStderr();
        std::future<int> exit_code = std::async(std::launch::async, [this, &args] {
            return run(args);
        });

        if (mountDirForUnmounting.is_initialized()) {
            boost::filesystem::path mountDir = *mountDirForUnmounting;
            std::future<bool> unmount_success = std::async(std::launch::async, [&mountDir] {
                int returncode = -1;
                while (returncode != 0) {
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
                return true;
            });

            if(std::future_status::ready != unmount_success.wait_for(std::chrono::seconds(10))) {
				testing::internal::GetCapturedStdout(); // stop capturing stdout
				testing::internal::GetCapturedStderr(); // stop capturing stderr

				std::cerr << "Unmount thread didn't finish";
				// The std::future destructor of a future created with std::async blocks until the future is ready.
				// so, instead of causing a deadlock, rather abort
				exit(EXIT_FAILURE);
            }
            EXPECT_TRUE(unmount_success.get()); // this also re-throws any potential exceptions
        }

        if(std::future_status::ready != exit_code.wait_for(std::chrono::seconds(10))) {
			testing::internal::GetCapturedStdout(); // stop capturing stdout
			testing::internal::GetCapturedStderr(); // stop capturing stderr
			
			std::cerr << "Filesystem thread didn't finish";
			// The std::future destructor of a future created with std::async blocks until the future is ready.
			// so, instead of causing a deadlock, rather abort
			exit(EXIT_FAILURE);
        }

		return {
			exit_code.get(),
			testing::internal::GetCapturedStdout(),
			testing::internal::GetCapturedStderr()
		};
    }
};

#endif
