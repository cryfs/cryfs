#pragma once
#ifndef MESSMER_CRYFS_PROGRAMOPTIONS_PROGRAMOPTIONS_H
#define MESSMER_CRYFS_PROGRAMOPTIONS_PROGRAMOPTIONS_H

#include <vector>
#include <string>
#include <boost/optional.hpp>
#include <cpp-utils/macros.h>
#include <boost/filesystem.hpp>

namespace cryfs {
    namespace program_options {
        class ProgramOptions final {
        public:
            ProgramOptions(const boost::filesystem::path &baseDir, const boost::filesystem::path &mountDir,
                           const boost::optional<boost::filesystem::path> &configFile,
                           bool foreground, const boost::optional<double> &unmountAfterIdleMinutes,
                           const boost::optional<boost::filesystem::path> &logFile,
                           const boost::optional<std::string> &cipher,
                           const boost::optional<uint32_t> &blocksizeBytes,
                           const std::vector<std::string> &fuseOptions);
            ProgramOptions(ProgramOptions &&rhs) = default;

            const boost::filesystem::path &baseDir() const;
            const boost::filesystem::path &mountDir() const;
            const boost::optional<boost::filesystem::path> &configFile() const;
            bool foreground() const;
            const boost::optional<std::string> &cipher() const;
            const boost::optional<uint32_t> &blocksizeBytes() const;
            const boost::optional<double> &unmountAfterIdleMinutes() const;
            const boost::optional<boost::filesystem::path> &logFile() const;
            const std::vector<std::string> &fuseOptions() const;

        private:
            boost::filesystem::path _baseDir;
            boost::filesystem::path _mountDir;
            boost::optional<boost::filesystem::path> _configFile;
            bool _foreground;
            boost::optional<std::string> _cipher;
            boost::optional<uint32_t> _blocksizeBytes;
            boost::optional<double> _unmountAfterIdleMinutes;
            boost::optional<boost::filesystem::path> _logFile;
            std::vector<std::string> _fuseOptions;

            DISALLOW_COPY_AND_ASSIGN(ProgramOptions);
        };
    }
}

#endif
