#include "parser.h"
#include <sstream>

using std::string;
using std::pair;
using std::tuple;
using std::tie;
using boost::optional;
using boost::none;
using std::istringstream;
using std::getline;

namespace gitversion {

    VersionInfo Parser::parse(const string &versionString) {
        VersionInfo result;
        string versionNumber;
        optional<string> versionInfo;
        tie(versionNumber, versionInfo) = _splitAtPlusSign(versionString);
        tie(result.majorVersion, result.minorVersion, result.versionTag) = _extractMajorMinorTag(versionNumber);
        result.isDevVersion = (versionInfo != none);
        result.isStableVersion = !result.isDevVersion && (result.versionTag == "" || result.versionTag == "stable");
        if (versionInfo != none && *versionInfo != "unknown") {
            result.gitCommitId = _extractGitCommitId(*versionInfo);
        } else {
            result.gitCommitId = "";
        }
        return result;
    }

    pair<string, optional<string>> Parser::_splitAtPlusSign(const string &versionString) {
        istringstream stream(versionString);
        string versionNumber;
        getline(stream, versionNumber, '+');
        if (!stream.good()) {
            return std::make_pair(versionNumber, none);
        } else {
            string versionInfo;
            getline(stream, versionInfo);
            return std::make_pair(versionNumber, versionInfo);
        }
    };

    tuple<string, string, string> Parser::_extractMajorMinorTag(const string &versionNumber) {
        istringstream stream(versionNumber);
        string major, minor, hotfix, tag;
        getline(stream, major, '.');
        if (!stream.good()) {
            minor = "0";
        } else {
            getline(stream, minor, '.');
        }
        if (!stream.good()) {
            hotfix = "0";
        } else {
            getline(stream, hotfix, '-');
        }
        if (!stream.good()) {
            tag = "";
        } else {
            getline(stream, tag);
        }
        return std::make_tuple(major, minor, tag);
    };

    string Parser::_extractGitCommitId(const string &versionInfo) {
        istringstream stream(versionInfo);
        string commitsSinceTag;
        getline(stream, commitsSinceTag, '.');
        if (!stream.good()) {
            throw std::logic_error("Invalid version information: Missing delimiter after commitsSinceTag.");
        }
        string gitCommitId;
        getline(stream, gitCommitId, '.');
        if (gitCommitId[0] != 'g') {
            throw std::logic_error("Invalid version information: Git commit id component doesn't start with 'g'.");
        }
        return gitCommitId.substr(1);
    }

}