#pragma once
#ifndef MESSMER_CRYFS_PROGRAMOPTIONS_PROGRAMOPTIONS_H
#define MESSMER_CRYFS_PROGRAMOPTIONS_PROGRAMOPTIONS_H

#include <vector>
#include <string>
#include <boost/optional.hpp>
#include <messmer/cpp-utils/macros.h>
#include <boost/filesystem.hpp>

namespace cryfs {
    namespace program_options {
        class ProgramOptions final {
        public:
            ProgramOptions(const boost::filesystem::path &baseDir, const boost::filesystem::path &mountDir,
                           const boost::optional<boost::filesystem::path> &configFile,
                           bool foreground, const boost::optional<double> &unmountAfterIdleMinutes,
                           const boost::optional<boost::filesystem::path> &logFile,
                           const boost::optional<std::string> &cipher, const boost::optional<std::string> &extPass,
                           const std::vector<char *> &fuseOptions);
            ProgramOptions(ProgramOptions &&rhs);
            ~ProgramOptions();

            const boost::filesystem::path &baseDir() const;
            boost::filesystem::path mountDir() const;
            const boost::optional<boost::filesystem::path> &configFile() const;
            bool foreground() const;
            const boost::optional<std::string> &cipher() const;
            const boost::optional<double> &unmountAfterIdleMinutes() const;
            const boost::optional<boost::filesystem::path> &logFile() const;
            const boost::optional<std::string> &extPass() const;
            const std::vector<char *> &fuseOptions() const;

        private:
            boost::filesystem::path _baseDir;
            char *_mountDir;
            boost::optional<boost::filesystem::path> _configFile;
            bool _foreground;
            boost::optional<std::string> _cipher;
            boost::optional<double> _unmountAfterIdleMinutes;
            boost::optional<boost::filesystem::path> _logFile;
            boost::optional<std::string> _extPass;
            std::vector<char *> _fuseOptions;

            DISALLOW_COPY_AND_ASSIGN(ProgramOptions);
        };
    }
}

#endif
