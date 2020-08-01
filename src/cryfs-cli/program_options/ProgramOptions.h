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
                           bool foreground, bool allowFilesystemUpgrade, bool allowReplacedFilesystem,
                           bool createMissingBasedir, bool createMissingMountpoint,
                           boost::optional<double> unmountAfterIdleMinutes,
                           boost::optional<boost::filesystem::path> logFile,
                           boost::optional<std::string> cipher,
                           boost::optional<uint32_t> blocksizeBytes,
                           bool allowIntegrityViolations,
                           boost::optional<bool> missingBlockIsIntegrityViolation,
                           std::vector<std::string> fuseOptions);
            ProgramOptions(ProgramOptions &&rhs) = default;

            const boost::filesystem::path &baseDir() const;
            const boost::filesystem::path &mountDir() const;
            const boost::optional<boost::filesystem::path> &configFile() const;
            bool foreground() const;
            bool allowFilesystemUpgrade() const;
            bool allowReplacedFilesystem() const;
            bool createMissingBasedir() const;
            bool createMissingMountpoint() const;
            const boost::optional<double> &unmountAfterIdleMinutes() const;
            const boost::optional<boost::filesystem::path> &logFile() const;
            const boost::optional<std::string> &cipher() const;
            const boost::optional<uint32_t> &blocksizeBytes() const;
            bool allowIntegrityViolations() const;
            const boost::optional<bool> &missingBlockIsIntegrityViolation() const;
            const std::vector<std::string> &fuseOptions() const;
			bool mountDirIsDriveLetter() const;

        private:
            boost::filesystem::path _baseDir; // this is always absolute
            boost::filesystem::path _mountDir; // this is absolute iff !_mountDirIsDriveLetter
			boost::optional<boost::filesystem::path> _configFile;
            bool _foreground;
            bool _allowFilesystemUpgrade;
            bool _allowReplacedFilesystem;
            bool _createMissingBasedir;
            bool _createMissingMountpoint;
            boost::optional<double> _unmountAfterIdleMinutes;
            boost::optional<boost::filesystem::path> _logFile;
            boost::optional<std::string> _cipher;
            boost::optional<uint32_t> _blocksizeBytes;
            bool _allowIntegrityViolations;
            boost::optional<bool> _missingBlockIsIntegrityViolation;
            std::vector<std::string> _fuseOptions;
			bool _mountDirIsDriveLetter;

            DISALLOW_COPY_AND_ASSIGN(ProgramOptions);
        };
    }
}

#endif
