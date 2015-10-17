#pragma once
#ifndef MESSMER_CRYFS_PROGRAMOPTIONS_PROGRAMOPTIONS_H
#define MESSMER_CRYFS_PROGRAMOPTIONS_PROGRAMOPTIONS_H

#include <vector>
#include <string>
#include <boost/optional.hpp>

namespace cryfs {
    namespace program_options {
        class ProgramOptions final {
        public:
            ProgramOptions(const std::string &baseDir, const std::string &mountDir, const std::string &configFile,
                           bool foreground, const boost::optional<std::string> &logFile,
                           const std::vector<char *> &fuseOptions);
            ~ProgramOptions();

            const std::string &baseDir() const;
            std::string mountDir() const;
            const std::string &configFile() const;
            bool foreground() const;
            const boost::optional<std::string> logFile() const;
            const std::vector<char *> &fuseOptions() const;

        private:
            std::string _baseDir;
            char *_mountDir;
            std::string _configFile;
            bool _foreground;
            boost::optional<std::string> _logFile;
            std::vector<char *> _fuseOptions;
        };
    }
}

#endif
