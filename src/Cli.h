#pragma once
#ifndef MESSMER_CRYFS_CLI_H
#define MESSMER_CRYFS_CLI_H

#include "program_options/ProgramOptions.h"
#include "config/CryConfigFile.h"
#include <boost/filesystem/path.hpp>
#include <messmer/cpp-utils/tempfile/TempFile.h>

namespace cryfs {
    class Cli final {
    public:
        int main(int argc, char *argv[]);

    private:
        static void _runFilesystem(const program_options::ProgramOptions &options);
        static CryConfigFile _loadOrCreateConfig(const program_options::ProgramOptions &options);
        static boost::filesystem::path _determineConfigFile(const program_options::ProgramOptions &options);
        static void _goToBackgroundIfSpecified(const program_options::ProgramOptions &options);
        static std::string _askPassword();
        static bool _checkPassword(const std::string &password);
        static void _showVersion();
        static void _initLogfile(const program_options::ProgramOptions &options);
        static void _sanityChecks(const program_options::ProgramOptions &options);
        static void _checkMountdirDoesntContainBasedir(const program_options::ProgramOptions &options);
        static bool _pathContains(const boost::filesystem::path &parent, const boost::filesystem::path &child);
        static void _checkBasedirAccessible(const program_options::ProgramOptions &options);
        static std::shared_ptr<cpputils::TempFile> _checkBasedirWriteable(const program_options::ProgramOptions &options);
        static void _checkBasedirReadable(const program_options::ProgramOptions &options, std::shared_ptr<cpputils::TempFile> tempfile);
    };
}

#endif
