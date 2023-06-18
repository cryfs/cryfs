#pragma once
#ifndef MESSMER_CPPUTILS_PROCESS_SUBPROCESS_H
#define MESSMER_CPPUTILS_PROCESS_SUBPROCESS_H

#include <string>
#include <vector>
#include <stdexcept>
#include <boost/filesystem/path.hpp>
#include "../macros.h"

namespace cpputils
{
    struct SubprocessResult final
    {
        std::string output_stdout;
        std::string output_stderr;
        int exitcode;
    };

    struct SubprocessError final : public std::runtime_error
    {
        SubprocessError(std::string msg) : std::runtime_error(std::move(msg)) {}
    };

    class Subprocess final
    {
    public:
        static SubprocessResult call(const char *command, const std::vector<std::string> &args, const std::string& input);
        static SubprocessResult call(const boost::filesystem::path &executable, const std::vector<std::string> &args, const std::string& input);
        static SubprocessResult check_call(const char *command, const std::vector<std::string> &args, const std::string& input);
        static SubprocessResult check_call(const boost::filesystem::path &executable, const std::vector<std::string> &args, const std::string& input);

    private:
        DISALLOW_COPY_AND_ASSIGN(Subprocess);
    };
}

#endif
