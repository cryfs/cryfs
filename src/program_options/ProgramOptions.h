#ifndef CRYFS_PROGRAMOPTIONS_PROGRAMOPTIONS_H
#define CRYFS_PROGRAMOPTIONS_PROGRAMOPTIONS_H

#include <vector>
#include <string>

namespace cryfs {
    namespace program_options {
        class ProgramOptions {
        public:
            ProgramOptions(const std::string &baseDir, const std::string &mountDir, const std::string &configFile, bool foreground, const std::vector<char *> &fuseOptions);
            ~ProgramOptions();

            const std::string &baseDir() const;
            std::string mountDir() const;
            const std::string &configFile() const;
            bool foreground() const;
            const std::vector<char *> &fuseOptions() const;

        private:
            std::string _baseDir;
            char *_mountDir;
            std::string _configFile;
            bool _foreground;
            std::vector<char *> _fuseOptions;
        };
    }
}

#endif
