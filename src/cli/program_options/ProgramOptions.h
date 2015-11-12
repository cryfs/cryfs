#pragma once
#ifndef MESSMER_CRYFS_PROGRAMOPTIONS_PROGRAMOPTIONS_H
#define MESSMER_CRYFS_PROGRAMOPTIONS_PROGRAMOPTIONS_H

#include <vector>
#include <string>
#include <boost/optional.hpp>
#include <messmer/cpp-utils/macros.h>

namespace cryfs {
    namespace program_options {
        class ProgramOptions final {
        public:
            ProgramOptions(const std::string &baseDir, const std::string &mountDir, const boost::optional<std::string> &configFile,
                           bool foreground, const boost::optional<unsigned int> &unmountAfterIdleMinutes,
                           const boost::optional<std::string> &logFile, const boost::optional<std::string> &cipher,
                           const boost::optional<std::string> &extPass, const std::vector<char *> &fuseOptions);
            ProgramOptions(ProgramOptions &&rhs);
            ~ProgramOptions();

            const std::string &baseDir() const;
            std::string mountDir() const;
            const boost::optional<std::string> &configFile() const;
            bool foreground() const;
            const boost::optional<std::string> &cipher() const;
            const boost::optional<unsigned int> &unmountAfterIdleMinutes() const;
            const boost::optional<std::string> &logFile() const;
            const boost::optional<std::string> &extPass() const;
            const std::vector<char *> &fuseOptions() const;

        private:
            std::string _baseDir;
            char *_mountDir;
            boost::optional<std::string> _configFile;
            bool _foreground;
            boost::optional<std::string> _cipher;
            boost::optional<unsigned int> _unmountAfterIdleMinutes;
            boost::optional<std::string> _logFile;
            boost::optional<std::string> _extPass;
            std::vector<char *> _fuseOptions;

            DISALLOW_COPY_AND_ASSIGN(ProgramOptions);
        };
    }
}

#endif
