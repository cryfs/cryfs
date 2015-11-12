#pragma once
#ifndef MESSMER_CRYFS_PROGRAMOPTIONS_UTILS_H
#define MESSMER_CRYFS_PROGRAMOPTIONS_UTILS_H

#include <utility>
#include <vector>

namespace cryfs {
    namespace program_options {
        /**
         * Splits an array of program options into two arrays of program options, split at a double dash '--' option.
         * It will not simply split the array, but it will also prepend options[0] to the second array,
         * since as a valid program options array, the second array should contain the executable name.
         */
        std::pair<std::vector<char*>, std::vector<char*>> splitAtDoubleDash(const std::vector<char*> &options);
    }
}

#endif
