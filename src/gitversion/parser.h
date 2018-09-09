#pragma once
#ifndef GITVERSION_PARSER_H
#define GITVERSION_PARSER_H

#include <boost/optional.hpp>
#include <string>
#include <utility>

namespace gitversion {
    struct VersionInfo {
        bool isDevVersion = false;
        bool isStableVersion = false;
        std::string versionTag;
        std::string gitCommitId;
        std::string majorVersion;
        std::string minorVersion;
        std::string hotfixVersion;
        unsigned int commitsSinceTag = 0;
    };

    class Parser final {
    public:
        static VersionInfo parse(const std::string &versionString);

    private:
        static std::pair<std::string, boost::optional<std::string>> _splitAt(const std::string &versionString, char delimiter);
        static std::tuple<std::string, std::string, std::string, std::string> _extractMajorMinorHotfixTag(const std::string &versionNumber);
        static std::tuple<std::string, std::string, std::string> _extractMajorMinorHotfix(const std::string &versionNumber);
        static std::tuple<std::string, unsigned long> _extractGitCommitIdAndCommitsSinceTag(const std::string &versionInfo);
    };
}

#endif
