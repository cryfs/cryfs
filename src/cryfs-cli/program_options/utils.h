#pragma once
#ifndef MESSMER_CRYFSCLI_PROGRAMOPTIONS_UTILS_H
#define MESSMER_CRYFSCLI_PROGRAMOPTIONS_UTILS_H

#include <utility>
#include <vector>
#include <string>

namespace cryfs_cli {
    namespace program_options {
        /**
         * Splits an array of program options into two arrays of program options, split at a double dash '--' option.
         */
        std::pair<std::vector<std::string>, std::vector<std::string>> splitAtDoubleDash(const std::vector<std::string> &options);
    }
}

#endif
