#pragma once
#ifndef MESSMER_CRYFSCLI_PROGRAMOPTIONS_PROGRAMOPTIONS_H
#define MESSMER_CRYFSCLI_PROGRAMOPTIONS_PROGRAMOPTIONS_H

#include <vector>
#include <string>
#include <boost/optional.hpp>
#include <cpp-utils/macros.h>
#include <boost/filesystem.hpp>

namespace cryfs_cli {
    namespace program_options {
        class ProgramOptions final {
        public:
            ProgramOptions(boost::filesystem::path baseDir, boost::filesystem::path mountDir,
                           boost::optional<boost::filesystem::path> configFile,
                           bool foreground, bool allowFilesystemUpgrade, bool allowReplacedFilesystem, boost::optional<double> unmountAfterIdleMinutes,
                           boost::optional<boost::filesystem::path> logFile,
                           boost::optional<std::string> cipher,
                           boost::optional<uint32_t> blocksizeBytes,
                           bool allowIntegrityViolations,
                           boost::optional<bool> missingBlockIsIntegrityViolation,
                           std::vector<std::string> fuseOptions);
            ProgramOptions(ProgramOptions &&rhs) = default;

            const boost::filesystem::path &baseDir() const;
            const boost::filesystem::path &mountDir() const;
			bool mountDirIsDriveLetter() const;
            const boost::optional<boost::filesystem::path> &configFile() const;
            bool foreground() const;
            bool allowFilesystemUpgrade() const;
            bool allowReplacedFilesystem() const;
            const boost::optional<std::string> &cipher() const;
            const boost::optional<uint32_t> &blocksizeBytes() const;
            const boost::optional<double> &unmountAfterIdleMinutes() const;
            bool allowIntegrityViolations() const;
            const boost::optional<bool> &missingBlockIsIntegrityViolation() const;
            const boost::optional<boost::filesystem::path> &logFile() const;
            const std::vector<std::string> &fuseOptions() const;

        private:
			boost::optional<boost::filesystem::path> _configFile;
            boost::filesystem::path _baseDir; // this is always absolute
            boost::filesystem::path _mountDir; // this is absolute iff !_mountDirIsDriveLetter
			bool _mountDirIsDriveLetter;
            bool _foreground;
            bool _allowFilesystemUpgrade;
            bool _allowReplacedFilesystem;
            bool _allowIntegrityViolations;
            boost::optional<std::string> _cipher;
            boost::optional<uint32_t> _blocksizeBytes;
            boost::optional<double> _unmountAfterIdleMinutes;
            boost::optional<bool> _missingBlockIsIntegrityViolation;
            boost::optional<boost::filesystem::path> _logFile;
            std::vector<std::string> _fuseOptions;

            DISALLOW_COPY_AND_ASSIGN(ProgramOptions);
        };
    }
}

#endif
