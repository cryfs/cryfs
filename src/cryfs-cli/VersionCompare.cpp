#include "VersionCompare.h"

using std::string;

namespace cryfs {

    bool VersionCompare::isOlderThan(const string &v1, const string &v2) {
        return _isOlderThanStartingFrom(v1, v2, 0, 0);
    }

    bool VersionCompare::_isOlderThanStartingFrom(const string &v1, const string &v2, size_t startPos1, size_t startPos2) {
        if (startPos1 > v1.size() && startPos2 > v2.size()) {
            // All components are equal
            return false;
        }
        string componentStr1 = _extractComponent(v1, startPos1);
        string componentStr2 = _extractComponent(v2, startPos2);
        uint32_t component1 = _parseComponent(componentStr1);
        uint32_t component2 = _parseComponent(componentStr2);
        if (component1 == component2) {
            return _isOlderThanStartingFrom(v1, v2, startPos1 + componentStr1.size() + 1, startPos2 + componentStr2.size() + 1);
        }
        return component1 < component2;
    }

    string VersionCompare::_extractComponent(const string &version, size_t startPos) {
        if (startPos >= version.size()) {
            return "";
        }
        size_t found = version.find('.', startPos);
        if (found == string::npos) {
            return version.substr(startPos);
        }
        return version.substr(startPos, found-startPos);
    }

    uint32_t VersionCompare::_parseComponent(const string &component) {
        if (component == "" || component.substr(0, 3) == "dev") {
            return 0;
        }
        return std::stoul(component);
    }
}
