#pragma once
#ifndef MESSMER_CRYFS_TEST_CLI_TESTUTILS_CLITEST_H
#define MESSMER_CRYFS_TEST_CLI_TESTUTILS_CLITEST_H

#include <google/gtest/gtest.h>
#include <messmer/cpp-utils/tempfile/TempDir.h>
#include <messmer/cpp-utils/tempfile/TempFile.h>
#include "../../../src/Cli.h"

class CliTest : public ::testing::Test {
public:
    CliTest(): basedir(), mountdir(), logfile(), configfile(false) {}

    cpputils::TempDir basedir;
    cpputils::TempDir mountdir;
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

    void EXPECT_RUN_SUCCESS(std::vector<const char*> args) {
        //TODO
        /*EXPECT_EXIT(
            run(args),
            ::testing::ExitedWithCode(0),
            "Filesystem is running"
        );*/
        //TODO Then stop running cryfs process again
    }
};

#endif
