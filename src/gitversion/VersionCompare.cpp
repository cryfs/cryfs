#include "VersionCompare.h"
#include "parser.h"
#include <tuple>
#include <cstring>

using std::string;

namespace gitversion {

    bool VersionCompare::isOlderThan(const string &v1Str, const string &v2Str) {
        VersionInfo v1 = Parser::parse(v1Str);
        VersionInfo v2 = Parser::parse(v2Str);
        unsigned long v1_major = std::stoul(v1.majorVersion);
        unsigned long v2_major = std::stoul(v2.majorVersion);
        unsigned long v1_minor = std::stoul(v1.minorVersion);
        unsigned long v2_minor = std::stoul(v2.minorVersion);
        unsigned long v1_hotfix = std::stoul(v1.hotfixVersion);
        unsigned long v2_hotfix = std::stoul(v2.hotfixVersion);
        int versionTagCompare = strcmp(v1.versionTag.c_str(), v2.versionTag.c_str());
        return (v1_major < v2_major) || ((v1_major == v2_major) && (
                (v1_minor < v2_minor) || ((v1_minor == v2_minor) && (
                 (v1_hotfix < v2_hotfix) || ((v1_hotfix == v2_hotfix) && (
                  (0 > versionTagCompare) || ((0 == versionTagCompare) && (
                   (v1.commitsSinceTag < v2.commitsSinceTag)
        ))))))));
    }
}
