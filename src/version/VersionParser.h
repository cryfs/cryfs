#ifndef MESSMER_CRYFS_VERSIONPARSER_H
#define MESSMER_CRYFS_VERSIONPARSER_H

#include <stdexcept>
#include "Version.h"
#include <messmer/cpp-utils/constexpr/const_string.h>

namespace version {
    class VersionParser {
    public:
        static constexpr Version parse(const cpputils::const_string &tagName, unsigned int commitsSinceVersion,
                                       const cpputils::const_string &gitCommitId) {
            return Version(VersionParser::extractMajor(tagName),
                           VersionParser::extractMinor(tagName),
                           parseTag(extractTag(tagName)),
                           commitsSinceVersion,
                           gitCommitId
            );
        }

        static constexpr unsigned int extractMajor(const cpputils::const_string &input) {
            return input.parseUIntPrefix();
        }

        static constexpr unsigned int extractMinor(const cpputils::const_string &input) {
            return (input.dropUIntPrefix()[0] == '.') ? input.dropUIntPrefix().dropPrefix(1).parseUIntPrefix()
                                                      : throw std::logic_error(
                            "Minor version should be separated by a dot");
        }

        static constexpr cpputils::const_string extractTag(const cpputils::const_string &input) {
            return input.dropUIntPrefix().dropPrefix(1).dropUIntPrefix();
        }

        static constexpr VersionTag parseTag(const cpputils::const_string &input) {
            return (VersionTagToString(VersionTag::ALPHA) == input) ? VersionTag::ALPHA :
                   (VersionTagToString(VersionTag::BETA) == input) ? VersionTag::BETA :
                   (VersionTagToString(VersionTag::RC1) == input) ? VersionTag::RC1 :
                   (VersionTagToString(VersionTag::FINAL) == input) ? VersionTag::FINAL :
                   throw std::logic_error("Not a valid version tag");
        }
    };
}

#endif

