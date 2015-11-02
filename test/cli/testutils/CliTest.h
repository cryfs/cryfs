#pragma once
#ifndef MESSMER_CRYFS_TEST_CLI_TESTUTILS_CLITEST_H
#define MESSMER_CRYFS_TEST_CLI_TESTUTILS_CLITEST_H

#include <google/gtest/gtest.h>
#include <google/gmock/gmock.h>
#include <messmer/cpp-utils/tempfile/TempDir.h>
#include <messmer/cpp-utils/tempfile/TempFile.h>
#include "../../../src/Cli.h"
#include <messmer/cpp-utils/logging/logging.h>

class CliTest : public ::testing::Test {
public:
    CliTest(): _basedir(), _mountdir(), basedir(_basedir.path()), mountdir(_mountdir.path()), logfile(), configfile(false) {}

    cpputils::TempDir _basedir;
    cpputils::TempDir _mountdir;
    boost::filesystem::path basedir;
    boost::filesystem::path mountdir;
    cpputils::TempFile logfile;
    cpputils::TempFile configfile;

    void run(std::vector<const char*> args) {
        std::vector<char*> _args;
        _args.reserve(args.size()+1);
        _args.push_back(const_cast<char*>("cryfs"));
        for (const char *arg : args) {
            _args.push_back(const_cast<char*>(arg));
        }
        cryfs::Cli().main(_args.size(), _args.data());
    }

    void EXPECT_EXIT_WITH_HELP_MESSAGE(std::vector<const char*> args) {
        EXPECT_RUN_ERROR(args, "Usage");
    }

    void EXPECT_RUN_ERROR(std::vector<const char*> args, const char *message) {
        EXPECT_EXIT(
            run(args),
            ::testing::ExitedWithCode(1),
            message
        );
    }

    void EXPECT_RUN_SUCCESS(std::vector<const char*> args, const boost::filesystem::path &mountDir) {
        std::thread unmountThread([&mountDir] {
            int returncode = -1;
            while (returncode != 0) {
                returncode = system((std::string("fusermount -u ") + mountDir.c_str()).c_str());
                std::this_thread::sleep_for(std::chrono::milliseconds(50)); // TODO Is this the test case duration? Does a shorter interval make the test case faster?
            }
        });
        //testing::internal::CaptureStdout();
        //TODO Don't force foreground, but find a way to run it also in background.
        args.push_back("-f");
        run(args);
        unmountThread.join();
        //EXPECT_THAT(testing::internal::GetCapturedStdout(), testing::MatchesRegex(".*Mounting filesystem.*"));
    }
};

#endif
