#include "VersionCompare.h"
#include "parser.h"
#include <tuple>
#include <cstring>

using std::string;

namespace gitversion {

    bool VersionCompare::isOlderThan(const string &v1Str, const string &v2Str) {
        const VersionInfo v1 = Parser::parse(v1Str);
        const VersionInfo v2 = Parser::parse(v2Str);
        const unsigned long v1_major = std::stoul(v1.majorVersion);
        const unsigned long v2_major = std::stoul(v2.majorVersion);
        const unsigned long v1_minor = std::stoul(v1.minorVersion);
        const unsigned long v2_minor = std::stoul(v2.minorVersion);
        const unsigned long v1_hotfix = std::stoul(v1.hotfixVersion);
        const unsigned long v2_hotfix = std::stoul(v2.hotfixVersion);
        const int versionTagCompare = _versionTagCompare(v1.versionTag, v2.versionTag);
        return (v1_major < v2_major) || ((v1_major == v2_major) && (
                (v1_minor < v2_minor) || ((v1_minor == v2_minor) && (
                 (v1_hotfix < v2_hotfix) || ((v1_hotfix == v2_hotfix) && (
                  (0 > versionTagCompare) || ((0 == versionTagCompare) && (
                   (v1.commitsSinceTag < v2.commitsSinceTag)
        ))))))));
    }

    int VersionCompare::_versionTagCompare(const string &tag1, const string &tag2) {
        if (tag1 == "") {
            if (tag2 == "") {
                return 0;
            } else {
                return 1;
            }
        } else {
            if (tag2 == "") {
                return -1;
            } else {
                return strcmp(tag1.c_str(), tag2.c_str());
            }
        }
    }
}
