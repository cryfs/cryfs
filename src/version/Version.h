#ifndef MESSMER_CRYFS_VERSION_H
#define MESSMER_CRYFS_VERSION_H

#include <stdexcept>
#include <messmer/cpp-utils/constexpr/const_string.h>

namespace version {
    enum class VersionTag : unsigned char {
        ALPHA, BETA, RC1, FINAL
    };

    constexpr cpputils::const_string VersionTagToString(VersionTag tag) {
        return (tag == VersionTag::ALPHA) ? "alpha" :
               (tag == VersionTag::BETA) ? "beta" :
               (tag == VersionTag::RC1) ? "rc1" :
               (tag == VersionTag::FINAL) ? "" :
               throw std::logic_error("Unknown version tag");
    }

    class Version {
    public:
        constexpr Version(unsigned int major, unsigned int minor, VersionTag tag, unsigned int commitsSinceVersion,
                          const cpputils::const_string &gitCommitId)
                : _major(major), _minor(minor), _tag(tag), _commitsSinceVersion(commitsSinceVersion),
                  _gitCommitId(gitCommitId) { }

        constexpr unsigned int major() {
            return _major;
        }

        constexpr unsigned int minor() {
            return _minor;
        }

        constexpr VersionTag tag() {
            return _tag;
        }

        constexpr bool is_dev() {
            return _commitsSinceVersion != 0;
        }

        constexpr bool is_stable() {
            return (!is_dev()) && _tag == VersionTag::FINAL;
        }

        constexpr bool operator==(const Version &rhs) {
            return _major == rhs._major && _minor == rhs._minor && _tag == rhs._tag;
        }

        constexpr bool operator!=(const Version &rhs) {
            return !operator==(rhs);
        }

        std::string toString() const {
            if (is_dev()) {
                return _versionTagString() + "-dev" + std::to_string(_commitsSinceVersion) + "-" + _gitCommitId.toStdString();
            } else {
                return _versionTagString();
            }
        }

    private:

        std::string _versionTagString() const {
            return std::to_string(_major) + "." + std::to_string(_minor) + VersionTagToString(_tag).toStdString();
        }

        const unsigned int _major;
        const unsigned int _minor;
        const VersionTag _tag;
        const unsigned int _commitsSinceVersion;
        const cpputils::const_string _gitCommitId;
    };
}


#endif
