#pragma once
#ifndef MESSMER_CRYFS_CLI_H
#define MESSMER_CRYFS_CLI_H

#include "program_options/ProgramOptions.h"
#include <cryfs/config/CryConfigFile.h>
#include <boost/filesystem/path.hpp>
#include <cpp-utils/tempfile/TempFile.h>
#include <cpp-utils/io/Console.h>
#include <cpp-utils/random/RandomGenerator.h>
#include <cpp-utils/network/HttpClient.h>
#include <cryfs/filesystem/CryDevice.h>
#include "CallAfterTimeout.h"
#include <cryfs/config/CryConfigLoader.h>
#include <cryfs/ErrorCodes.h>

namespace cryfs {
    class Cli final {
    public:
        Cli(cpputils::RandomGenerator &keyGenerator, const cpputils::SCryptSettings& scryptSettings, std::shared_ptr<cpputils::Console> console);
        int main(int argc, const char *argv[], cpputils::unique_ref<cpputils::HttpClient> httpClient);

    private:
        void _checkForUpdates(cpputils::unique_ref<cpputils::HttpClient> httpClient);
        void _runFilesystem(const program_options::ProgramOptions &options);
        CryConfigLoader::ConfigLoadResult _loadOrCreateConfig(const program_options::ProgramOptions &options, const LocalStateDir& localStateDir);
        void _checkConfigIntegrity(const boost::filesystem::path& basedir, const LocalStateDir& localStateDir, const CryConfigFile& config, bool allowReplacedFilesystem);
        boost::optional<CryConfigLoader::ConfigLoadResult> _loadOrCreateConfigFile(boost::filesystem::path configFilePath, LocalStateDir localStateDir, const boost::optional<std::string> &cipher, const boost::optional<uint32_t> &blocksizeBytes, bool allowFilesystemUpgrade, const boost::optional<bool> &missingBlockIsIntegrityViolation, bool allowReplacedFilesystem);
        boost::filesystem::path _determineConfigFile(const program_options::ProgramOptions &options);
        static std::function<std::string()> _askPasswordForExistingFilesystem(std::shared_ptr<cpputils::Console> console);
        static std::function<std::string()> _askPasswordForNewFilesystem(std::shared_ptr<cpputils::Console> console);
        static std::function<std::string()> _askPasswordNoninteractive(std::shared_ptr<cpputils::Console> console);
        static bool _confirmPassword(cpputils::Console* console, const std::string &password);
        static bool _checkPassword(const std::string &password);
        void _showVersion(cpputils::unique_ref<cpputils::HttpClient> httpClient);
        void _initLogfile(const program_options::ProgramOptions &options);
        void _sanityChecks(const program_options::ProgramOptions &options);
        void _checkMountdirDoesntContainBasedir(const program_options::ProgramOptions &options);
        bool _pathContains(const boost::filesystem::path &parent, const boost::filesystem::path &child);
        void _checkDirAccessible(const boost::filesystem::path &dir, const std::string &name, ErrorCode errorCode);
        std::shared_ptr<cpputils::TempFile> _checkDirWriteable(const boost::filesystem::path &dir, const std::string &name, ErrorCode errorCode);
        void _checkDirReadable(const boost::filesystem::path &dir, std::shared_ptr<cpputils::TempFile> tempfile, const std::string &name, ErrorCode errorCode);
        boost::optional<cpputils::unique_ref<CallAfterTimeout>> _createIdleCallback(boost::optional<double> minutes, std::function<void()> callback);
        void _sanityCheckFilesystem(CryDevice *device);


        cpputils::RandomGenerator &_keyGenerator;
        cpputils::SCryptSettings _scryptSettings;
        std::shared_ptr<cpputils::Console> _console;
        bool _noninteractive;

        DISALLOW_COPY_AND_ASSIGN(Cli);
    };
}

#endif
