#pragma once
#ifndef MESSMER_CRYFS_PROGRAMOPTIONS_PARSER_H
#define MESSMER_CRYFS_PROGRAMOPTIONS_PARSER_H

#include "ProgramOptions.h"
#include <boost/program_options.hpp>

namespace cryfs {
    namespace program_options {
        class Parser {
        public:
            Parser(int argc, char *argv[]);

            ProgramOptions parse() const;

        private:
            static std::vector<char *> _argsToVector(int argc, char *argv[]);
            static void _addAllowedOptions(boost::program_options::options_description *desc);
            static void _addPositionalOptionForBaseDir(boost::program_options::options_description *desc,
                                                       boost::program_options::positional_options_description *positional);
            [[noreturn]] static void _showHelpAndExit();
            static boost::program_options::variables_map _parseOptionsOrShowHelp(const std::vector<char*> options);
            static boost::program_options::variables_map _parseOptions(const std::vector<char*> options);

            std::vector<char*> _options;
        };
    }
}

#endif
