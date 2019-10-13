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
        tie(versionNumber, versionInfo) = _splitAt(versionString, '+');
        tie(result.majorVersion, result.minorVersion, result.hotfixVersion, result.versionTag) = _extractMajorMinorHotfixTag(versionNumber);
        result.isDevVersion = (versionInfo != none);
        result.isStableVersion = !result.isDevVersion && (result.versionTag == "" || result.versionTag == "stable");
        if (versionInfo != none && *versionInfo != "unknown") {
            tie(result.gitCommitId, result.commitsSinceTag) = _extractGitCommitIdAndCommitsSinceTag(*versionInfo);
        } else {
            result.gitCommitId = "";
            result.commitsSinceTag = 0;
        }
        return result;
    }

    pair<string, optional<string>> Parser::_splitAt(const string &versionString, char delimiter) {
        istringstream stream(versionString);
        string versionNumber;
        getline(stream, versionNumber, delimiter);
        if (!stream.good()) {
            return std::make_pair(versionNumber, none);
        } else {
            string versionInfo;
            getline(stream, versionInfo);
            return std::make_pair(versionNumber, versionInfo);
        }
    }

    tuple<string, string, string, string> Parser::_extractMajorMinorHotfixTag(const string &versionNumber) {
        string majorMinorHotfix;
        optional<string> versionTag;
        tie(majorMinorHotfix, versionTag) = _splitAt(versionNumber, '-');
        string major, minor, hotfix;
        tie(major, minor, hotfix) = _extractMajorMinorHotfix(majorMinorHotfix);
        if (versionTag == none) {
            versionTag = "";
        }
        return std::make_tuple(major, minor, hotfix, *versionTag);
    }

    tuple<string, string, string> Parser::_extractMajorMinorHotfix(const string &versionNumber) {
        istringstream stream(versionNumber);
        string major, minor, hotfix;
        getline(stream, major, '.');
        if (!stream.good()) {
            minor = "0";
        } else {
            getline(stream, minor, '.');
        }
        if (!stream.good()) {
            hotfix = "0";
        } else {
            getline(stream, hotfix);
        }
        return std::make_tuple(major, minor, hotfix);
    };

    std::tuple<string, unsigned long> Parser::_extractGitCommitIdAndCommitsSinceTag(const string &versionInfo) {
        istringstream stream(versionInfo);
        string commitsSinceTag;
        getline(stream, commitsSinceTag, '.');
        if (!stream.good()) {
            throw std::logic_error("Invalid version information: Missing delimiter after commitsSinceTag (versionInfo: "+versionInfo+").");
        }
        string gitCommitId;
        getline(stream, gitCommitId, '.');
        if (gitCommitId[0] != 'g') {
            throw std::logic_error("Invalid version information: Git commit id component doesn't start with 'g' (versionInfo: "+versionInfo+").");
        }
        return std::make_tuple(gitCommitId.substr(1), std::stoul(commitsSinceTag));
    }

}
