#pragma once
#ifndef MESSMER_CRYFS_TEST_CLI_TESTUTILS_CLITEST_H
#define MESSMER_CRYFS_TEST_CLI_TESTUTILS_CLITEST_H

#include <google/gtest/gtest.h>
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

    class _UnmountFilesystemInDestructor final {
    public:
        _UnmountFilesystemInDestructor(const boost::filesystem::path &baseDir) :_baseDir(baseDir) {}
        ~_UnmountFilesystemInDestructor() {
            if (0 != system((std::string("fusermount -u ")+_baseDir.c_str()).c_str())) {
                cpputils::logging::LOG(cpputils::logging::ERROR) << "Could not unmount cryfs";
            }
        }
    private:
        boost::filesystem::path _baseDir;
    };

    void EXPECT_RUN_SUCCESS(std::vector<const char*> args, const boost::filesystem::path &baseDir) {
        //TODO
        /*_UnmountFilesystemInDestructor raii(baseDir);
        EXPECT_EXIT(
            run(args),
            ::testing::ExitedWithCode(0),
            "Filesystem is running"
        );*/
    }
};

#endif
