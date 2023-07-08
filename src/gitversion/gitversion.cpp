#include "gitversion.h"
#include "parser.h"

using std::string;

namespace gitversion {

    const VersionInfo &parse() {
        static const VersionInfo versionInfo = Parser::parse(VersionString());
        return versionInfo;
    }

    bool IsDevVersion() {
        return parse().isDevVersion;
    }

    bool IsStableVersion() {
        return parse().isStableVersion;
    }

    string GitCommitId() {
        return parse().gitCommitId;
    }

    string MajorVersion() {
        return parse().majorVersion;
    }

    string MinorVersion() {
        return parse().minorVersion;
    }
}
