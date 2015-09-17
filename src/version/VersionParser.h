#ifndef MESSMER_CRYFS_VERSIONPARSER_H
#define MESSMER_CRYFS_VERSIONPARSER_H

#include <stdexcept>
#include <cstring>
#include "Version.h"

namespace version {
    class VersionParser {
    public:
        static constexpr Version parse(const char *input) {
            return Version(VersionParser::extractMajor(input),
                           VersionParser::extractMinor(input),
                           parseTag(extractTag(input)));
        }

        static constexpr unsigned int extractMajor(const char *input) {
            return parseNumber(input);
        }

        static constexpr unsigned int extractMinor(const char *input) {
            return (skipNumber(input)[0] == '.') ? parseNumber(skipNumber(input) + 1)
                                                 : throw std::logic_error("Minor version should be separated by a dot");
        }

        static constexpr unsigned int parseNumber(const char *input) {
            return (isDigit(input[0])) ? parseNumberBackwards(input + numDigits(input) - 1, numDigits(input))
                                       : throw std::logic_error("Not a valid number");
        }

        static constexpr unsigned int numDigits(const char *input) {
            return (isDigit(input[0])) ? (1 + numDigits(input + 1)) : 0;
        }

        static constexpr bool isDigit(char digit) {
            return digit == '0' || digit == '1' || digit == '2' || digit == '3' || digit == '4' || digit == '5' ||
                   digit == '6' || digit == '7' || digit == '8' || digit == '9';
        }

        static constexpr unsigned char parseDigit(char digit) {
            return (digit == '0') ? 0 :
                   (digit == '1') ? 1 :
                   (digit == '2') ? 2 :
                   (digit == '3') ? 3 :
                   (digit == '4') ? 4 :
                   (digit == '5') ? 5 :
                   (digit == '6') ? 6 :
                   (digit == '7') ? 7 :
                   (digit == '8') ? 8 :
                   (digit == '9') ? 9 :
                   throw std::logic_error("Not a valid digit");
        }

        static constexpr const char *extractTag(const char *input) {
            return skipNumber(skipNumber(input) + 1);
        }

        static constexpr VersionTag parseTag(const char *input) {
            return (0 == strcmp(VersionTagToString(VersionTag::ALPHA), input)) ? VersionTag::ALPHA :
                   (0 == strcmp(VersionTagToString(VersionTag::BETA), input)) ? VersionTag::BETA :
                   (0 == strcmp(VersionTagToString(VersionTag::RC1), input)) ? VersionTag::RC1 :
                   (0 == strcmp(VersionTagToString(VersionTag::FINAL), input)) ? VersionTag::FINAL :
                   throw std::logic_error("Not a valid version tag");
        }

        static constexpr const char *skipNumber(const char *input) {
            return (isDigit(input[0])) ? skipNumber(input + 1) : input;
        }

    private:
        static constexpr unsigned int parseNumberBackwards(const char *input, unsigned int numDigits) {
            return (numDigits == 0) ? 0 : (parseDigit(*input) + 10 * parseNumberBackwards(input - 1, numDigits - 1));
        }
    };
}

#endif

