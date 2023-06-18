#include "versionstring.h"

using std::string;

namespace gitversion {

    const string &VersionString() {
        static const string VERSION_STRING = GIT_VERSION_STRING; // GIT_VERSION_STRING is set by our CMake file as a macro
        return VERSION_STRING;
    }
}
