#include "parser.h"
#include <regex>

using std::string;
using std::pair;
using std::tuple;
using std::tie;
using std::regex;
using std::smatch;
using std::regex_match;
using boost::optional;
using boost::none;

namespace gitversion {

    VersionInfo Parser::parse(const string &versionString) {
        VersionInfo result;
        string versionNumber;
        optional<string> versionInfo;
        tie(versionNumber, versionInfo) = _splitAtPlusSign(versionString);
        tie(result.majorVersion, result.minorVersion, result.versionTag) = _extractMajorMinorTag(versionNumber);
        result.isDevVersion = (versionInfo != none);
        result.isStableVersion = !result.isDevVersion && (result.versionTag == "" || result.versionTag == "stable");
        if (versionInfo != none) {
            result.gitCommitId = _extractGitCommitId(*versionInfo);
        } else {
            result.gitCommitId = "";
        }
        return result;
    }

    pair<string, optional<string>> Parser::_splitAtPlusSign(const string &versionString) {
        regex splitRegex("^([^+]+)(\\+([^+]+))?$");
        smatch match;
        regex_match(versionString, match, splitRegex);
        if(match[0] != versionString) {
            throw std::logic_error("First match group should be whole string");
        }
        if(match.size() != 4) {
            throw std::logic_error("Wrong number of match groups");
        }
        if (match[2].matched) {
            return std::make_pair(match[1], optional<string>(match[3]));
        } else {
            return std::make_pair(match[1], none);
        }
    };

    tuple<string, string, string> Parser::_extractMajorMinorTag(const string &versionNumber) {
        regex splitRegex("^([0-9]+)\\.([0-9]+)\\.[0-9\\.]*(-(.*))?$");
        smatch match;
        regex_match(versionNumber, match, splitRegex);
        if(match[0] != versionNumber) {
            throw std::logic_error("First match group should be whole string");
        }
        if(match.size() != 5) {
            throw std::logic_error("Wrong number of match groups");
        }
        if(match[3].matched) {
            return std::make_tuple(match[1], match[2], match[4]);
        } else {
            return std::make_tuple(match[1], match[2], "");
        }
    };

    string Parser::_extractGitCommitId(const string &versionInfo) {
        regex extractRegex("^[0-9]+\\.g([0-9a-f]+)(\\..*)?$");
        smatch match;
        regex_match(versionInfo, match, extractRegex);
        if(match[0] != versionInfo) {
            throw std::logic_error("First match group should be whole string");
        }
        if(match.size() != 3) {
            throw std::logic_error("Wrong number of match groups");
        }
        return match[1];
    }

}