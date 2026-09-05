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

        /**
         * Removes the 'nonempty' mount option from a list of fuse options and returns whether it was there.
         *
         * libfuse 2 refused to mount over a directory that already had files in it unless this option was
         * given. libfuse 3.0 removed both the check and the option and left the decision to the file system,
         * and libfuse 3.10.2 brought the option back as a no-op. CryFS therefore does the check itself and
         * consumes the option here, so it behaves the same way on every libfuse 3 release.
         *
         * Options have to be preceded by a '-o', i.e. {..., "-o", "nonempty", ...}, and several of them can be
         * comma-concatenated, i.e. {..., "-o", "nonempty,allow_other", ...}. A '-o' left without a value is
         * removed as well, because libfuse rejects it.
         */
        bool extractNonemptyOption(std::vector<std::string> *fuseOptions);
    }
}

#endif
