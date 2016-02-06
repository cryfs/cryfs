#pragma once
#ifndef MESSMER_CRYFS_TEST_CLI_TESTUTILS_CLITEST_H
#define MESSMER_CRYFS_TEST_CLI_TESTUTILS_CLITEST_H

#include <google/gtest/gtest.h>
#include <google/gmock/gmock.h>
#include <messmer/cpp-utils/tempfile/TempDir.h>
#include <messmer/cpp-utils/tempfile/TempFile.h>
#include "../../../src/cli/Cli.h"
#include "../../../src/cli/VersionChecker.h"
#include <messmer/cpp-utils/logging/logging.h>
#include <messmer/cpp-utils/process/subprocess.h>
#include <messmer/cpp-utils/network/FakeHttpClient.h>
#include "../../testutils/MockConsole.h"

class CliTest : public ::testing::Test {
public:
    CliTest(): _basedir(), _mountdir(), basedir(_basedir.path()), mountdir(_mountdir.path()), logfile(), configfile(false), console(std::make_shared<MockConsole>()) {}

    cpputils::TempDir _basedir;
    cpputils::TempDir _mountdir;
    boost::filesystem::path basedir;
    boost::filesystem::path mountdir;
    cpputils::TempFile logfile;
    cpputils::TempFile configfile;
    std::shared_ptr<MockConsole> console;

    std::shared_ptr<cpputils::HttpClient> _httpClient() {
        std::shared_ptr<cpputils::FakeHttpClient> httpClient = std::make_shared<cpputils::FakeHttpClient>();
        httpClient->addWebsite("https://www.cryfs.org/version_info.json", "{\"version_info\":{\"current\":\"0.8.5\"}}");
        return httpClient;
    }

    void run(std::vector<const char*> args) {
        std::vector<char*> _args;
        _args.reserve(args.size()+1);
        _args.push_back(const_cast<char*>("cryfs"));
        for (const char *arg : args) {
            _args.push_back(const_cast<char*>(arg));
        }
        auto &keyGenerator = cpputils::Random::PseudoRandom();
        cryfs::Cli(keyGenerator, cpputils::SCrypt::TestSettings, console, _httpClient()).main(_args.size(), _args.data());
    }

    void EXPECT_EXIT_WITH_HELP_MESSAGE(std::vector<const char*> args, const std::string &message = "") {
        EXPECT_RUN_ERROR(args, (message+".*Usage").c_str());
    }

    void EXPECT_RUN_ERROR(std::vector<const char*> args, const char *message) {
        EXPECT_EXIT(
            run(args),
            ::testing::ExitedWithCode(1),
            message
        );
    }

    void EXPECT_RUN_SUCCESS(std::vector<const char*> args, const boost::filesystem::path &mountDir) {
        //TODO Make this work when run in background
        ASSERT(std::find(args.begin(), args.end(), string("-f")) != args.end(), "Currently only works if run in foreground");
        std::thread unmountThread([&mountDir] {
            int returncode = -1;
            while (returncode != 0) {
                returncode = cpputils::Subprocess::callAndGetReturnCode(std::string("fusermount -u ") + mountDir.c_str() + " 2>/dev/null");
                std::this_thread::sleep_for(std::chrono::milliseconds(50)); // TODO Is this the test case duration? Does a shorter interval make the test case faster?
            }
        });
        testing::internal::CaptureStdout();
        run(args);
        unmountThread.join();
        EXPECT_THAT(testing::internal::GetCapturedStdout(), testing::MatchesRegex(".*Mounting filesystem.*"));
    }
};

#endif
