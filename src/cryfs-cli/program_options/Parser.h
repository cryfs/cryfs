#pragma once
#ifndef MESSMER_CRYFSCLI_PROGRAMOPTIONS_PARSER_H
#define MESSMER_CRYFSCLI_PROGRAMOPTIONS_PARSER_H

#include "ProgramOptions.h"
#include <boost/program_options.hpp>
#include <cryfs/ErrorCodes.h>

namespace cryfs_cli {
    namespace program_options {
        class Parser final {
        public:
            Parser(int argc, const char **argv);

            ProgramOptions parse(const std::vector<std::string> &supportedCiphers) const;

        private:
            static std::vector<std::string> _argsToVector(int argc, const char **argv);
            static std::vector<const char*> _to_const_char_vector(const std::vector<std::string> &options);
            static void _addAllowedOptions(boost::program_options::options_description *desc);
            static void _addPositionalOptionForBaseDir(boost::program_options::options_description *desc,
                                                       boost::program_options::positional_options_description *positional);
            static void _showHelp();
            [[noreturn]] static void _showHelpAndExit(const std::string& message, cryfs::ErrorCode errorCode);
            [[noreturn]] static void _showCiphersAndExit(const std::vector<std::string> &supportedCiphers);
            [[noreturn]] static void _showVersionAndExit();
            static boost::program_options::variables_map _parseOptionsOrShowHelp(const std::vector<std::string> &options, const std::vector<std::string> &supportedCiphers);
            static boost::program_options::variables_map _parseOptions(const std::vector<std::string> &options, const std::vector<std::string> &supportedCiphers);
            static void _checkValidCipher(const std::string &cipher, const std::vector<std::string> &supportedCiphers);

            std::vector<std::string> _options;

            DISALLOW_COPY_AND_ASSIGN(Parser);
        };
    }
}

#endif
