#pragma once
#ifndef GITVERSION_VERSIONCOMPARE_H
#define GITVERSION_VERSIONCOMPARE_H

#include <string>

namespace gitversion {
    class VersionCompare {
    public:
        static bool isOlderThan(const std::string &v1, const std::string &v2);
    };
}

#endif
