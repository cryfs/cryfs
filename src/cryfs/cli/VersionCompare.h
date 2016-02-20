#pragma once
#ifndef MESSMER_CRYFS_CLI_VERSIONCOMPARE_H
#define MESSMER_CRYFS_CLI_VERSIONCOMPARE_H

#include <string>

namespace cryfs {
    class VersionCompare {
    public:
        static bool isOlderThan(const std::string &v1, const std::string &v2);

    private:
        static bool _isOlderThanStartingFrom(const std::string &v1, const std::string &v2, size_t startPos1, size_t startPos2);
        static std::string _extractComponent(const std::string &version, size_t startPos);
        static uint32_t _parseComponent(const std::string &component);
    };
}

#endif
