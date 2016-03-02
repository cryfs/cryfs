#pragma once
#ifndef GITVERSION_PARSER_H
#define GITVERSION_PARSER_H

#include <boost/optional.hpp>
#include <string>
#include <utility>

namespace gitversion {
    struct VersionInfo {
        bool isDevVersion;
        bool isStableVersion;
        std::string versionTag;
        std::string gitCommitId;
        std::string majorVersion;
        std::string minorVersion;
    };

    class Parser final {
    public:
        static VersionInfo parse(const std::string &versionString);

    private:
        static std::pair<std::string, boost::optional<std::string>> _splitAtPlusSign(const std::string &versionString);
        static std::tuple<std::string, std::string, std::string> _extractMajorMinorTag(const std::string &versionNumber);
        static std::string _extractGitCommitId(const std::string &versionInfo);
    };
}

#endif
